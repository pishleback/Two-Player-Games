use super::*;

#[derive(Debug)]
pub struct AlphaBeta<G: GameLogic + Send> {
    game: Option<Game<G>>,
    score_quality_generator: ScoreQualityGenerator,
    score_quality: ScoreQuality,
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
        let mut score_quality_generator = ScoreQuality::generate();
        let score_quality = score_quality_generator.next().unwrap();
        Self {
            game: None,
            score_quality_generator,
            score_quality,
            search_findings: None,
            persistent: Arc::new(Mutex::new(AlphaBetaPersistent::new())),
        }
    }

    fn set_game(&mut self, game: Game<G>) {
        self.score_quality_generator = ScoreQuality::generate();
        self.score_quality = self.score_quality_generator.next().unwrap();
        self.search_findings = None;
        self.game = Some(game);
    }

    fn think(&mut self, max_time: chrono::TimeDelta) {
        if let Some(game) = &self.game {
            let stop = chrono::Utc::now() + max_time;
            let mut state = game.state().clone();
            while !stop.stop() {
                let mut node_count = 0;
                if let Ok((score, best_move_at_depth)) =
                    negamax_alphabeta_score::<chrono::DateTime<chrono::Utc>, _>(
                        stop,
                        0,
                        game.logic(),
                        &mut state,
                        self.persistent.clone(),
                        self.score_quality,
                        0,
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
                        || current_best.as_ref().unwrap().score_quality < self.score_quality
                    {
                        *current_best = Some(SearchFindings {
                            score_quality: self.score_quality,
                            best_move: best_move_at_depth,
                            node_count: total_node_count,
                        });
                        log::info!(
                            "\
                        \tScore={:?} Depth={} MaxQuiescenceDepth={} Nodes={total_node_count}",
                            score,
                            self.score_quality.depth,
                            self.score_quality.quiescence_depth,
                        );
                    }
                    self.score_quality = self.score_quality_generator.next().unwrap();
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
