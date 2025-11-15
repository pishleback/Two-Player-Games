use super::*;

#[derive(Debug)]
pub struct AlphaBeta<G: GameLogic + Send> {
    game: Option<Game<G>>,
    depth: usize,
    max_quiescence_depth: usize,
    search_findings: Option<SearchFindings<G>>,
    persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
}

impl StopCondition for chrono::DateTime<chrono::Utc> {
    fn stop(&self) -> bool {
        chrono::Utc::now() > *self
    }
}

impl<G: GameLogic + Send> Ai<G> for AlphaBeta<G> {
    fn new() -> Self {
        Self {
            game: None,
            depth: 1,
            max_quiescence_depth: 1,
            search_findings: None,
            persistent: Arc::new(Mutex::new(AlphaBetaPersistent::new())),
        }
    }

    fn set_game(&mut self, game: Game<G>) {
        self.depth = 1;
        self.max_quiescence_depth = 1;
        self.search_findings = None;
        self.game = Some(game);
    }

    fn think(&mut self, max_time: chrono::TimeDelta) {
        if let Some(game) = &self.game {
            let stop = chrono::Utc::now() + max_time;
            let mut state = game.state().clone();
            while !stop.stop() && self.max_quiescence_depth <= 100 {
                let mut node_count = 0;
                if let Ok((score, best_move_at_depth)) =
                    negamax_alphabeta_score::<chrono::DateTime<chrono::Utc>, _>(
                        stop,
                        0,
                        game.logic(),
                        &mut state,
                        self.persistent.clone(),
                        self.depth as isize,
                        0,
                        self.max_quiescence_depth,
                        &mut node_count,
                        WithNegInf::NegInf,
                        WithPosInf::PosInf,
                    )
                {
                    let current_best = &mut self.search_findings;
                    let total_node_count = current_best
                        .as_ref()
                        .map_or(0, |sf: &SearchFindings<G>| sf.node_count)
                        + node_count;
                    if current_best.is_none()
                        || current_best.as_ref().unwrap().depth < self.depth
                        || current_best.as_ref().unwrap().max_quiescence_depth
                            < self.max_quiescence_depth
                    {
                        *current_best = Some(SearchFindings {
                            depth: self.depth,
                            max_quiescence_depth: self.max_quiescence_depth,
                            best_move: best_move_at_depth,
                            node_count: total_node_count,
                        });
                        log::info!(
                            "\
                        \tScore={:?} Depth={} MaxQuiescenceDepth={} Nodes={total_node_count}",
                            score,
                            self.depth,
                            self.max_quiescence_depth,
                        );
                    }
                    self.max_quiescence_depth *= 2;
                }
            }

            while !stop.stop() && self.depth <= 100 {
                let mut node_count = 0;
                if let Ok((score, best_move_at_depth)) =
                    negamax_alphabeta_score::<chrono::DateTime<chrono::Utc>, _>(
                        stop,
                        0,
                        game.logic(),
                        &mut state,
                        self.persistent.clone(),
                        self.depth as isize,
                        0,
                        self.max_quiescence_depth,
                        &mut node_count,
                        WithNegInf::NegInf,
                        WithPosInf::PosInf,
                    )
                {
                    let current_best = &mut self.search_findings;
                    let total_node_count = current_best
                        .as_ref()
                        .map_or(0, |sf: &SearchFindings<G>| sf.node_count)
                        + node_count;
                    if current_best.is_none()
                        || current_best.as_ref().unwrap().depth < self.depth
                        || current_best.as_ref().unwrap().max_quiescence_depth
                            < self.max_quiescence_depth
                    {
                        *current_best = Some(SearchFindings {
                            depth: self.depth,
                            max_quiescence_depth: self.max_quiescence_depth,
                            best_move: best_move_at_depth,
                            node_count: total_node_count,
                        });
                        log::info!(
                            "\
\tScore={:?} Depth={} Nodes={total_node_count}",
                            score,
                            self.depth
                        );
                    }
                    self.depth += 1;
                }
            }
        }
    }

    fn best_move(&self) -> Option<G::Move> {
        self.search_findings
            .as_ref()
            .and_then(|search_findings| search_findings.best_move.clone())
    }
}
