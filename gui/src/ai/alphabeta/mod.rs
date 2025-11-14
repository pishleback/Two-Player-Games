use std::sync::{Arc, Mutex};

use crate::game::{RelScore, State, StateIdent, WithNegInf, WithPosInf};
use crate::{
    ai::Ai,
    game::{Game, GameLogic},
};

#[cfg(not(target_arch = "wasm32"))]
mod multithreaded;
#[cfg(not(target_arch = "wasm32"))]
pub use multithreaded::*;

pub mod singlethreaded;
#[cfg(target_arch = "wasm32")]
pub use singlethreaded::*;

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
    state: G::StateIdent,
    score: Option<TranspositionTableEntry<G>>,
}

impl<G: GameLogic + Send> TranspositionTableItem<G> {
    fn blank(state: G::StateIdent) -> Self {
        Self { state, score: None }
    }
}

#[derive(Debug)]
struct TranspositionTable<G: GameLogic + Send> {
    n: u64,
    entries: Vec<Option<TranspositionTableItem<G>>>,
}

impl<G: GameLogic + Send> TranspositionTable<G> {
    fn new(n: u64) -> Self {
        debug_assert!(n <= 64);
        Self {
            n,
            entries: (0..(1usize << n)).map(|_| None).collect(),
        }
    }

    fn idx_hash(&self, state: &G::StateIdent) -> usize {
        let hash64 = state.hash64();
        (hash64 & ((1 << self.n) - 1)) as usize
    }

    fn maybe_get(&self, state: G::StateIdent) -> Option<&Option<TranspositionTableEntry<G>>> {
        let idx = self.idx_hash(&state);
        let entry_opt = &self.entries[idx];
        if let Some(entry) = entry_opt {
            #[allow(clippy::if_same_then_else)]
            if entry.state.hash64() != state.hash64() {
                return None;
            } else if entry.state != state {
                return None;
            }
        } else {
            return None;
        }
        Some(&entry_opt.as_ref().unwrap().score)
    }

    fn get(&mut self, state: G::StateIdent) -> &mut Option<TranspositionTableEntry<G>> {
        let idx = self.idx_hash(&state);
        let entry_opt = &mut self.entries[idx];
        if let Some(entry) = entry_opt {
            #[allow(clippy::if_same_then_else)]
            if entry.state.hash64() != state.hash64() {
                *entry_opt = Some(TranspositionTableItem::blank(state));
            } else if entry.state != state {
                *entry_opt = Some(TranspositionTableItem::blank(state));
            }
        } else {
            *entry_opt = Some(TranspositionTableItem::blank(state));
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
        log::info!("Create Transposition Table");
        let available_bytes = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut sys = sysinfo::System::new();
                sys.refresh_memory();
                sys.available_memory()
            }
            #[cfg(target_arch = "wasm32")]
            {
                // Use at most 3.5 GB on wasm32
                3584 * 1024 * 1024
            }
        };

        log::info!("\tAvailable space {} MB", available_bytes / (1024 * 1024));
        let available_bytes = (available_bytes * 90) / 100;
        let bytes_per_entry = std::mem::size_of::<Option<TranspositionTableItem<G>>>() as u64;
        let max_tt_entries = available_bytes / bytes_per_entry;
        let mut n = 0;
        while (1 << (n + 1)) <= max_tt_entries {
            n += 1;
        }
        log::info!(
            "\tAllocating {} entries in {} MB...",
            (1 << n),
            (bytes_per_entry * (1 << n)) / (1024 * 1024),
        );
        let p = Self {
            transpositions: TranspositionTable::new(n),
        };
        log::info!("\tDone");
        p
    }
}

trait StopCondition: Clone {
    fn stop(&self) -> bool;
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn negamax_alphabeta_score<S: StopCondition, G: GameLogic + Send>(
    stop: S,
    thread_num: usize,
    logic: &G,
    state: &mut G::State,
    persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
    target_depth: isize,
    depth_from_root: usize,
    max_quiescence_depth: usize,
    node_count: &mut usize,
    mut alpha: WithNegInf<RelScore<G::HeuristicScore>>,
    beta: WithPosInf<RelScore<G::HeuristicScore>>,
) -> Result<(RelScore<G::HeuristicScore>, Option<G::Move>), ()> {
    if stop.stop() {
        return Err(());
    }
    *node_count += 1;
    let player = logic.turn(state);

    let orig_alpha = alpha.clone();

    // Transposition Table lookup
    /*
    The condition `depth_from_root >= 2` is added so that the search is not blind (via transposition table entries) to stumbling into a draw when in a winning position.
    The problem is explained here https://talkchess.com/viewtopic.php?t=20080
     */
    let probable_best_move = if depth_from_root >= 2
        && let Some(Some(tt_entry)) = persistent
            .lock()
            .unwrap()
            .transpositions
            .maybe_get(state.clone().ident())
        && tt_entry.depth >= target_depth
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
    let (moves, mut best_score) = if target_depth <= 0 {
        let stand_pat = logic.score(state).into_rel(player);
        let stand_pat_with_neg_inf = WithNegInf::Finite(stand_pat.clone());
        if alpha < stand_pat_with_neg_inf {
            alpha = stand_pat_with_neg_inf.clone();
        }
        if alpha >= beta {
            return Ok((stand_pat, None));
        }
        if target_depth < -(max_quiescence_depth as isize) {
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

    if depth_from_root == 2 {
        state.set_ignore_repetitions(true);
    }
    let mut best_move = None;
    for mv in ordered_moves {
        #[cfg(debug_assertions)]
        let state_before = (*state).clone();
        logic.make_move(state, &mv);
        debug_assert_ne!(logic.turn(state), player);
        let (score, _) = negamax_alphabeta_score::<S, G>(
            stop.clone(),
            thread_num,
            logic,
            state,
            persistent.clone(),
            target_depth - 1,
            depth_from_root + 1,
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
    if depth_from_root == 2 {
        state.set_ignore_repetitions(false);
    }

    // Transposition Table store
    let mut persistent = persistent.lock().unwrap();
    let tt_entry_opt = persistent.transpositions.get(state.clone().ident());
    *tt_entry_opt = Some(TranspositionTableEntry {
        depth: target_depth,
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
