use crate::game::{RelScore, WithNegInf, WithPosInf};
use crate::{
    ai::Ai,
    game::{Game, GameLogic},
};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

#[derive(Debug)]
enum TranspositionTableEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug)]
struct TranspositionTableEntry<G: GameLogic + Send> {
    depth: isize,
    max_quiescence_depth: usize,
    score: RelScore<G::HeuristicScore>,
    best_move: Option<G::Move>,
    flag: TranspositionTableEntryFlag,
}

#[derive(Debug)]
struct TranspositionTableItem<G: GameLogic + Send> {
    state: G::State,
    score: Option<TranspositionTableEntry<G>>,
}

impl<G: GameLogic + Send> TranspositionTableItem<G> {
    fn blank(state: G::State) -> Self {
        Self { state, score: None }
    }
}

#[derive(Debug)]
struct TranspositionTable<G: GameLogic + Send> {
    n: usize,
    entries: Vec<Option<TranspositionTableItem<G>>>,
}

impl<G: GameLogic + Send> TranspositionTable<G> {
    fn new(n: usize) -> Self {
        debug_assert!(n <= 64);
        Self {
            n,
            entries: (0..(1usize << n)).map(|_| None).collect(),
        }
    }

    fn hash(&self, logic: &G, state: &G::State) -> usize {
        let hash64 = logic.hash_state(state);
        (hash64 & ((1 << self.n) - 1)) as usize
    }

    fn maybe_get(
        &self,
        logic: &G,
        state: &G::State,
    ) -> Option<&Option<TranspositionTableEntry<G>>> {
        let idx = self.hash(logic, state);
        let entry_opt = &self.entries[idx];
        if let Some(entry) = entry_opt {
            #[allow(clippy::if_same_then_else)]
            if logic.hash_state(&entry.state) != logic.hash_state(state) {
                return None;
            } else if entry.state != *state {
                return None;
            }
        } else {
            return None;
        }
        Some(&entry_opt.as_ref().unwrap().score)
    }

    fn get(&mut self, logic: &G, state: &G::State) -> &mut Option<TranspositionTableEntry<G>> {
        let idx = self.hash(logic, state);
        let entry_opt = &mut self.entries[idx];
        if let Some(entry) = entry_opt {
            #[allow(clippy::if_same_then_else)]
            if logic.hash_state(&entry.state) != logic.hash_state(state) {
                *entry_opt = Some(TranspositionTableItem::blank(state.clone()));
            } else if entry.state != *state {
                *entry_opt = Some(TranspositionTableItem::blank(state.clone()));
            }
        } else {
            *entry_opt = Some(TranspositionTableItem::blank(state.clone()));
        }
        &mut entry_opt.as_mut().unwrap().score
    }
}

#[derive(Debug)]
struct AlphaBetaPersistent<G: GameLogic + Send> {
    transpositions: TranspositionTable<G>,
}

impl<G: GameLogic + Send> AlphaBetaPersistent<G> {
    fn new() -> Self {
        Self {
            transpositions: TranspositionTable::new(26),
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn negamax_alphabeta_score<G: GameLogic + Send>(
    stop: Arc<AtomicBool>,
    thread_num: usize,
    logic: &G,
    state: &mut G::State,
    persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
    depth: isize,
    max_quiescence_depth: usize,
    node_count: &mut usize,
    mut alpha: WithNegInf<RelScore<G::HeuristicScore>>,
    beta: WithPosInf<RelScore<G::HeuristicScore>>,
) -> Result<(RelScore<G::HeuristicScore>, Option<G::Move>), ()> {
    if stop.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(());
    }
    *node_count += 1;
    let player = logic.turn(state);

    let orig_alpha = alpha.clone();

    // Transposition Table lookup
    let probable_best_move = if let Some(Some(tt_entry)) = persistent
        .lock()
        .unwrap()
        .transpositions
        .maybe_get(logic, state)
        && tt_entry.depth >= depth
        && tt_entry.max_quiescence_depth >= max_quiescence_depth
    {
        match tt_entry.flag {
            TranspositionTableEntryFlag::Exact => {
                return Ok((tt_entry.score.clone(), tt_entry.best_move.clone()));
            }
            TranspositionTableEntryFlag::LowerBound => {
                if WithPosInf::Finite(tt_entry.score.clone()) >= beta {
                    return Ok((tt_entry.score.clone(), tt_entry.best_move.clone()));
                }
            }
            TranspositionTableEntryFlag::UpperBound => {
                if WithNegInf::Finite(tt_entry.score.clone()) <= alpha {
                    return Ok((tt_entry.score.clone(), tt_entry.best_move.clone()));
                }
            }
        }
        tt_entry.best_move.clone()
    } else {
        None
    };

    // Alpha-Beta search
    let (moves, mut best_score) = if depth <= 0 {
        let stand_pat = logic.score(state).into_rel(player);
        let stand_pat_with_neg_inf = WithNegInf::Finite(stand_pat.clone());
        if alpha < stand_pat_with_neg_inf {
            alpha = stand_pat_with_neg_inf.clone();
        }
        if alpha >= beta {
            return Ok((stand_pat, None));
        }
        if depth < -(max_quiescence_depth as isize) {
            return Ok((stand_pat, None));
        }
        (
            logic.generate_quiescence_moves(state),
            stand_pat_with_neg_inf,
        )
    } else {
        (logic.generate_moves(state), WithNegInf::NegInf)
    };

    if moves.is_empty() {
        return Ok((logic.score(state).into_rel(player), None));
    }

    let ordered_moves = if let Some(probable_best_move) = probable_best_move {
        vec![probable_best_move.clone()]
            .into_iter()
            .chain({
                let mut moves = moves
                    .into_iter()
                    .filter(|mv| mv != &probable_best_move)
                    .collect::<Vec<_>>();

                // Shuffle so different threads look at different things
                fn shuffle<T>(vec: &mut [T], mut seed: usize) {
                    fn next_u32(seed: &mut usize) -> u32 {
                        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                        (*seed >> 16) as u32
                    }
                    let len = vec.len();
                    for i in (1..len).rev() {
                        let j = (next_u32(&mut seed) as usize) % (i + 1);
                        vec.swap(i, j);
                    }
                }
                shuffle(&mut moves, thread_num);

                moves
            })
            .collect()
    } else {
        moves
    };

    let mut best_move = None;
    for mv in ordered_moves {
        #[cfg(debug_assertions)]
        let state_before = (*state).clone();
        logic.make_move(state, &mv);
        debug_assert_ne!(logic.turn(state), player);
        let (score, _) = negamax_alphabeta_score(
            stop.clone(),
            thread_num,
            logic,
            state,
            persistent.clone(),
            depth - 1,
            max_quiescence_depth,
            node_count,
            -beta.clone(),
            -alpha.clone(),
        )?;
        let score = -score;
        let score = score.inc_time();
        logic.unmake_move(state, &mv);
        #[cfg(debug_assertions)]
        assert_eq!(*state, state_before);
        let score = WithNegInf::Finite(score);
        if best_score < score {
            best_score = score.clone();
            best_move = Some(mv);
        }
        if alpha < score {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    // Transposition Table store
    let mut persistent = persistent.lock().unwrap();
    let tt_entry_opt = persistent.transpositions.get(logic, state);
    *tt_entry_opt = Some(TranspositionTableEntry {
        depth,
        max_quiescence_depth,
        score: best_score.clone().unwrap_finite(),
        best_move: best_move.clone(),
        flag: {
            if best_score <= orig_alpha {
                TranspositionTableEntryFlag::UpperBound
            } else if best_score >= beta {
                TranspositionTableEntryFlag::LowerBound
            } else {
                TranspositionTableEntryFlag::Exact
            }
        },
    });

    Ok((best_score.unwrap_finite(), best_move))
}

#[derive(Debug)]
struct SearchFindings<G: GameLogic> {
    depth: usize,
    max_quiescence_depth: usize,
    best_move: Option<G::Move>,
    node_count: usize,
}

#[derive(Debug)]
struct AlphaBetaSearch<G: GameLogic + Send> {
    stop: Arc<AtomicBool>,
    search_findings: Arc<Mutex<Option<SearchFindings<G>>>>, // (depth, node count, move)
    persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
}

impl<G: GameLogic + Send> Drop for AlphaBetaSearch<G> {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<G: GameLogic + Send> AlphaBetaSearch<G> {
    fn new(game: Game<G>, persistent: Arc<Mutex<AlphaBetaPersistent<G>>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let search_findings = Arc::new(Mutex::new(None));

        println!("Start Thinking...");

        for i in 0..num_cpus::get() {
            let stop = stop.clone();
            let persistent = persistent.clone();
            let search_findings = search_findings.clone();
            let logic = game.logic().clone();
            let mut state = game.state().clone();
            std::thread::spawn(move || {
                let mut depth: usize = 1;
                let mut max_quiescence_depth: usize = 1;
                while !stop.load(std::sync::atomic::Ordering::Relaxed)
                    && max_quiescence_depth <= 100
                {
                    let mut node_count = 0;
                    if let Ok((score, best_move_at_depth)) = negamax_alphabeta_score(
                        stop.clone(),
                        i,
                        &logic,
                        &mut state,
                        persistent.clone(),
                        depth as isize,
                        max_quiescence_depth,
                        &mut node_count,
                        WithNegInf::NegInf,
                        WithPosInf::PosInf,
                    ) {
                        let mut current_best = search_findings.lock().unwrap();
                        let total_node_count = current_best
                            .as_ref()
                            .map_or(0, |sf: &SearchFindings<G>| sf.node_count)
                            + node_count;
                        if current_best.is_none()
                            || current_best.as_ref().unwrap().depth < depth
                            || current_best.as_ref().unwrap().max_quiescence_depth
                                < max_quiescence_depth
                        {
                            *current_best = Some(SearchFindings {
                                depth,
                                max_quiescence_depth,
                                best_move: best_move_at_depth,
                                node_count: total_node_count,
                            });
                            println!(
                                "\
Depth={depth} MaxQuiescenceDepth={max_quiescence_depth} Nodes={total_node_count} Score={:?}",
                                score
                            );
                        }
                    }
                    max_quiescence_depth *= 2;
                }

                while !stop.load(std::sync::atomic::Ordering::Relaxed) && depth <= 100 {
                    let mut node_count = 0;
                    if let Ok((score, best_move_at_depth)) = negamax_alphabeta_score(
                        stop.clone(),
                        i,
                        &logic,
                        &mut state,
                        persistent.clone(),
                        depth as isize,
                        max_quiescence_depth,
                        &mut node_count,
                        WithNegInf::NegInf,
                        WithPosInf::PosInf,
                    ) {
                        let mut current_best = search_findings.lock().unwrap();
                        let total_node_count = current_best
                            .as_ref()
                            .map_or(0, |sf: &SearchFindings<G>| sf.node_count)
                            + node_count;
                        if current_best.is_none()
                            || current_best.as_ref().unwrap().depth < depth
                            || current_best.as_ref().unwrap().max_quiescence_depth
                                < max_quiescence_depth
                        {
                            *current_best = Some(SearchFindings {
                                depth,
                                max_quiescence_depth,
                                best_move: best_move_at_depth,
                                node_count: total_node_count,
                            });
                            println!(
                                "\
Depth={depth} Nodes={total_node_count} Score={:?}",
                                score
                            );
                        }
                    }

                    depth += 1;
                }
            });
        }

        Self {
            stop: stop.clone(),
            search_findings: search_findings.clone(),
            persistent: persistent.clone(),
        }
    }

    fn end(self) -> Arc<Mutex<AlphaBetaPersistent<G>>> {
        self.persistent.clone()
    }

    fn best_move(&self) -> Option<G::Move> {
        self.search_findings
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|search_findings| search_findings.best_move.clone())
    }
}

#[allow(private_interfaces)]
#[derive(Debug)]
pub enum AlphaBeta<G: GameLogic + Send> {
    Temp,
    Idle {
        persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
    },
    Running {
        search: AlphaBetaSearch<G>,
    },
}

impl<G: GameLogic + Send> Ai<G> for AlphaBeta<G> {
    fn new() -> Self {
        Self::Idle {
            persistent: Arc::new(Mutex::new(AlphaBetaPersistent::new())),
        }
    }

    fn set_game(&mut self, game: Game<G>) {
        let old = std::mem::replace(self, AlphaBeta::Temp);
        *self = match old {
            AlphaBeta::Idle { persistent } => Self::Running {
                search: AlphaBetaSearch::new(game, persistent),
            },
            AlphaBeta::Running { search } => Self::Running {
                search: AlphaBetaSearch::new(game, search.end()),
            },
            AlphaBeta::Temp => unreachable!(),
        };
    }

    fn think(&mut self, _max_time: chrono::TimeDelta) {}

    fn best_move(&self) -> Option<G::Move> {
        match self {
            AlphaBeta::Idle { .. } => None,
            AlphaBeta::Running { search } => search.best_move(),
            AlphaBeta::Temp => unreachable!(),
        }
    }
}
