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
struct AlphaBetaPersistent<G: GameLogic + Send> {
    _g: PhantomData<G>,
}

impl<G: GameLogic + Send> Default for AlphaBetaPersistent<G> {
    fn default() -> Self {
        Self {
            _g: Default::default(),
        }
    }
}

// Scores returned are relative to `player`
fn negamax_alphabeta_score<G: GameLogic>(
    stop: Arc<AtomicBool>,
    logic: &G,
    player: Player,
    state: &mut G::State,
    depth: isize,
    depth_reached: &mut isize,
    node_count: &mut usize,
    mut alpha: G::Score,
    beta: G::Score,
    principal_variation: Option<Vec<G::Move>>,
) -> Result<(G::Score, Vec<G::Move>), ()> {
    if stop.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(());
    }

    *depth_reached = std::cmp::min(*depth_reached, depth);
    *node_count += 1;

    let (mut moves, mut best_score) = if depth <= 0 {
        (
            logic.generate_quiescence_moves(player, state),
            match player {
                Player::First => logic.score(state),
                Player::Second => -logic.score(state),
            },
        )
    } else {
        (logic.generate_moves(player, state), G::Score::neg_inf())
    };

    if moves.is_empty() {
        // Absolute score - not relative to player
        Ok((
            match player {
                Player::First => logic.score(state),
                Player::Second => -logic.score(state),
            },
            vec![],
        ))
    } else {
        if let Some(mut principal_variation_moves) = principal_variation
            && let Some(principal_move) = principal_variation_moves.pop()
        {
            moves = [principal_move.clone()]
                .into_iter()
                .chain(moves.into_iter().filter(|mv| *mv != principal_move))
                .collect();
        }

        let mut best_moves = vec![];
        for mv in moves {
            logic.make_move(state, &mv);
            let (score, mut moves) = negamax_alphabeta_score(
                stop.clone(),
                logic,
                player.flip(),
                state,
                depth - 1,
                depth_reached,
                node_count,
                -beta.clone(),
                -alpha.clone(),
                None,
            )?;
            let score = -score;
            logic.unmake_move(state, &mv);
            if best_score < score {
                best_score = score.clone();
                moves.push(mv);
                best_moves = moves;
            }
            if alpha < score {
                alpha = score.clone();
            }
            if beta <= alpha {
                // prune
                break;
            }
        }
        Ok((best_score, best_moves))
    }
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
    fn new(game: Game<G>, persistent: AlphaBetaPersistent<G>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));

        let best_move = Arc::new(Mutex::new(None));

        let logic = game.logic().clone();
        let player = game.turn();
        let mut state = game.state().clone();

        Self {
            game,
            stop: stop.clone(),
            best_move: best_move.clone(),
            run_thread: ManuallyDrop::new(std::thread::spawn(move || {
                println!("Start Thinking");
                let mut depth = 1;
                let mut principal_variation = None;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut depth_reached = depth;
                    let mut node_count = 0;
                    if let Ok((score, new_principal_variation)) = negamax_alphabeta_score(
                        stop.clone(),
                        &logic,
                        player,
                        &mut state,
                        depth,
                        &mut depth_reached,
                        &mut node_count,
                        G::Score::neg_inf(),
                        G::Score::pos_inf(),
                        principal_variation.clone(),
                    ) {
                        *best_move.lock().unwrap() = new_principal_variation.last().cloned();
                        principal_variation = Some(new_principal_variation);
                        println!(
                            "Done at depth={depth}. Max quiescence depth={}. Nodes={node_count}. Score={:?}",
                            depth - depth_reached,
                            score
                        );
                    } else {
                        println!(
                            "Done at depth={depth}. Max quiescence depth={}. Nodes={node_count}.",
                            depth - depth_reached
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
    Idle { persistent: AlphaBetaPersistent<G> },
    Running { search: AlphaBetaSearch<G> },
}

impl<G: GameLogic + Send> Default for AlphaBeta<G> {
    fn default() -> Self {
        Self::Idle {
            persistent: Default::default(),
        }
    }
}

impl<G: GameLogic + Send> Ai<G> for AlphaBeta<G> {
    fn set_game(&mut self, game: Game<G>) {
        let old = std::mem::take(self);
        *self = match old {
            AlphaBeta::Idle { persistent } => Self::Running {
                search: AlphaBetaSearch::new(game, persistent),
            },
            AlphaBeta::Running { search } => Self::Running {
                search: AlphaBetaSearch::new(game, search.end()),
            },
        };
    }

    fn think(&mut self, max_time: chrono::TimeDelta) {}

    fn best_move(&self) -> Option<G::Move> {
        match self {
            AlphaBeta::Idle { .. } => None,
            AlphaBeta::Running { search } => search.best_move(),
        }
    }
}
