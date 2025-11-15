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
    search_findings: Arc<Mutex<Option<SearchFindings<G>>>>,
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

        let n = num_cpus::get();
        log::info!("Thinking on {} Threads...", n);
        for i in 0..n {
            let stop = stop.clone();
            let persistent = persistent.clone();
            let search_findings = search_findings.clone();
            let logic = game.logic().clone();
            let mut state = game.state().clone();
            std::thread::spawn(move || {
                for score_quality in ScoreQuality::generate() {
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
                        let total_node_count = current_best
                            .as_ref()
                            .map_or(0, |sf: &SearchFindings<G>| sf.node_count)
                            + node_count;
                        if current_best.is_none()
                            || current_best.as_ref().unwrap().score_quality < score_quality
                        {
                            *current_best = Some(SearchFindings {
                                score_quality,
                                best_move: best_move_at_depth,
                                node_count: total_node_count,
                            });
                            log::info!(
                                "\
\tScore={:?} Depth={} MaxQuiescenceDepth={} Nodes={total_node_count}",
                                score,
                                score_quality.depth,
                                score_quality.quiescence_depth
                            );
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
