use std::sync::atomic::AtomicBool;

use super::*;

impl StopCondition for Arc<AtomicBool> {
    fn stop(&self) -> bool {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct AlphaBetaSearch<G: GameLogic + Send> {
    stop: Arc<AtomicBool>,
    search_findings: Arc<Mutex<AllSearchFindings<G>>>,
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
        let search_findings = Arc::new(Mutex::new(AllSearchFindings::new()));

        let n = num_cpus::get();
        log::info!("Thinking on {} Threads...", n);
        for i in 0..n {
            let stop = stop.clone();
            let persistent = persistent.clone();
            let search_findings = search_findings.clone();
            let logic = game.logic().clone();
            let total_node_count = Arc::new(Mutex::<usize>::new(0));
            let mut state = game.state().clone();
            std::thread::spawn(move || {
                let pvec = match i {
                    0 => PvExtensionCounter::new(0, 1),
                    1 => PvExtensionCounter::new(usize::MAX, usize::MAX),
                    2 => PvExtensionCounter::new(0, 2),
                    3 => PvExtensionCounter::new(1, 2),
                    4 => PvExtensionCounter::new(0, 3),
                    5 => PvExtensionCounter::new(1, 3),
                    6 => PvExtensionCounter::new(2, 3),
                    _ => PvExtensionCounter::new(usize::MAX, usize::MAX),
                };

                for score_quality in ScoreQuality::generate(pvec) {
                    if stop.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    let mut node_count = 0;
                    if let Ok((score, best_move_at_depth)) =
                        negamax_alphabeta_score::<Arc<AtomicBool>, _>(
                            stop.clone(),
                            i,
                            &logic,
                            &mut state,
                            persistent.clone(),
                            score_quality,
                            0,
                            &mut node_count,
                            WithNegInf::NegInf,
                            WithPosInf::PosInf,
                        )
                    {
                        let mut current_best = search_findings.lock().unwrap();
                        let mut total_node_count = total_node_count.lock().unwrap();
                        *total_node_count += node_count;
                        if let Some(best_move) = best_move_at_depth {
                            current_best.update(SearchFindings {
                                score_quality,
                                score,
                                best_move,
                            });
                        }
                    }
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

    fn best_moves(&self) -> Vec<(String, G::Move)> {
        match self {
            AlphaBeta::Idle { .. } => vec![],
            AlphaBeta::Running { search } => search.search_findings.lock().unwrap().best_moves(),
            AlphaBeta::Temp => unreachable!(),
        }
    }
}
