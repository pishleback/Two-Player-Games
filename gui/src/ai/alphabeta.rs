use crate::game::Score;
use crate::{
    ai::Ai,
    game::{Game, GameLogic, Player},
};
use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread::JoinHandle,
};

#[derive(Debug)]
enum TranspositionTableEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug)]
struct TranspositionTableEntry<G: GameLogic + Send> {
    depth: isize,
    score: G::Score,
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

    fn get(&mut self, logic: &G, state: &G::State) -> &mut Option<TranspositionTableEntry<G>> {
        let idx = self.hash(logic, state);
        let entry_opt = &mut self.entries[idx];
        if let Some(entry) = entry_opt {
            if logic.hash_state(&entry.state) != logic.hash_state(state) {
                // println!("Different Hash");
                *entry_opt = Some(TranspositionTableItem::blank(state.clone()));
            } else if entry.state != *state {
                // println!("Same Hash Different State");
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

// Scores returned are relative to `player`
fn negamax_alphabeta_score<G: GameLogic + Send>(
    stop: Arc<AtomicBool>,
    logic: &G,
    state: &mut G::State,
    transpositions: &mut TranspositionTable<G>,
    depth: isize,
    depth_reached: &mut isize,
    node_count: &mut usize,
    mut alpha: G::Score,
    beta: G::Score,
) -> Result<(G::Score, Option<G::Move>), ()> {
    if stop.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(());
    }
    *depth_reached = std::cmp::min(depth, *depth_reached);
    *node_count += 1;
    let player = logic.turn(state);

    let orig_alpha = alpha.clone();

    // Transposition Table lookup
    let probable_best_move = if let Some(tt_entry) = transpositions.get(logic, state)
        && tt_entry.depth >= depth
    {
        match tt_entry.flag {
            TranspositionTableEntryFlag::Exact => {
                return Ok((tt_entry.score.clone(), tt_entry.best_move.clone()));
            }
            TranspositionTableEntryFlag::LowerBound => {
                if tt_entry.score >= beta {
                    return Ok((tt_entry.score.clone(), tt_entry.best_move.clone()));
                }
            }
            TranspositionTableEntryFlag::UpperBound => {
                if tt_entry.score <= alpha {
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
        let stand_pat = match player {
            Player::First => logic.score(state),
            Player::Second => -logic.score(state),
        };
        if alpha < stand_pat {
            alpha = stand_pat.clone();
        }
        if alpha >= beta {
            return Ok((stand_pat, None));
        }
        (logic.generate_quiescence_moves(state), stand_pat.clone())
    } else {
        (logic.generate_moves(state), G::Score::neg_inf())
    };

    if moves.is_empty() {
        return Ok((
            match player {
                Player::First => logic.score(state),
                Player::Second => -logic.score(state),
            },
            None,
        ));
    }

    let ordered_moves = if let Some(probable_best_move) = probable_best_move {
        vec![probable_best_move.clone()]
            .into_iter()
            .chain(moves.into_iter().filter(|mv| mv != &probable_best_move))
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
            logic,
            state,
            transpositions,
            depth - 1,
            depth_reached,
            node_count,
            -beta.clone(),
            -alpha.clone(),
        )?;
        let score = -score;
        logic.unmake_move(state, &mv);
        #[cfg(debug_assertions)]
        assert_eq!(*state, state_before);
        if best_score < score {
            best_score = score.clone();
            best_move = Some(mv);
        }
        if alpha < score {
            alpha = score.clone();
        }
        if alpha >= beta {
            break;
        }
    }

    // Transposition Table store
    let tt_entry_opt = transpositions.get(logic, state);
    *tt_entry_opt = Some(TranspositionTableEntry {
        depth,
        score: best_score.clone(),
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

    Ok((best_score, best_move))
}

#[derive(Debug)]
struct AlphaBetaSearch<G: GameLogic + Send> {
    game: Game<G>,
    stop: Arc<AtomicBool>,
    best_move: Arc<Mutex<Option<G::Move>>>,
    run_thread: ManuallyDrop<JoinHandle<AlphaBetaPersistent<G>>>,
}

impl<G: GameLogic + Send> Drop for AlphaBetaSearch<G> {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<G: GameLogic + Send> AlphaBetaSearch<G> {
    fn new(game: Game<G>, mut persistent: AlphaBetaPersistent<G>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));

        let best_move = Arc::new(Mutex::new(None));

        let logic = game.logic().clone();
        let mut state = game.state().clone();

        Self {
            game,
            stop: stop.clone(),
            best_move: best_move.clone(),
            run_thread: ManuallyDrop::new(std::thread::spawn(move || {
                println!("Start Thinking {:?}", logic.turn(&state));
                let mut depth = 1;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut depth_reached = 0;
                    let mut node_count = 0;
                    if let Ok((score, best_move_at_depth)) = negamax_alphabeta_score(
                        stop.clone(),
                        &logic,
                        &mut state,
                        &mut persistent.transpositions,
                        depth,
                        &mut depth_reached,
                        &mut node_count,
                        G::Score::neg_inf(),
                        G::Score::pos_inf(),
                    ) {
                        *best_move.lock().unwrap() = best_move_at_depth;
                        println!(
                            "Done at depth={depth}. Max quiescence depth={}. Nodes={node_count}. Score={:?}",
                            depth - depth_reached,
                            score
                        );
                    }

                    depth += 1;
                }
                persistent
            })),
        }
    }

    fn end(mut self) -> AlphaBetaPersistent<G> {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        unsafe { ManuallyDrop::take(&mut self.run_thread) }
            .join()
            .unwrap()
    }

    fn best_move(&self) -> Option<G::Move> {
        self.best_move.lock().unwrap().as_ref().cloned()
    }
}

#[allow(private_interfaces)]
#[derive(Debug)]
pub enum AlphaBeta<G: GameLogic + Send> {
    Temp,
    Idle { persistent: AlphaBetaPersistent<G> },
    Running { search: AlphaBetaSearch<G> },
}

impl<G: GameLogic + Send> Ai<G> for AlphaBeta<G> {
    fn new() -> Self {
        Self::Idle {
            persistent: AlphaBetaPersistent::new(),
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

    fn think(&mut self, max_time: chrono::TimeDelta) {}

    fn best_move(&self) -> Option<G::Move> {
        match self {
            AlphaBeta::Idle { .. } => None,
            AlphaBeta::Running { search } => search.best_move(),
            AlphaBeta::Temp => unreachable!(),
        }
    }
}
