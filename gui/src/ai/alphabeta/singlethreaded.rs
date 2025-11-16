use super::*;

#[derive(Debug)]
pub struct AlphaBeta<G: GameLogic + Send> {
    game: Option<Game<G>>,
    pv_extension_counter: PvExtensionCounter,
    score_quality_generator: ScoreQualityGenerator,
    score_quality: Option<ScoreQuality>,
    node_count: usize,
    search_findings: AllSearchFindings<G>,
    persistent: Arc<Mutex<AlphaBetaPersistent<G>>>,
}

impl StopCondition for chrono::DateTime<chrono::Utc> {
    fn stop(&self) -> bool {
        chrono::Utc::now() > *self
    }
}

impl<G: GameLogic + Send> Ai<G> for AlphaBeta<G> {
    fn new() -> Self {
        let pv_extension_counter = PvExtensionCounter::new(usize::MAX, usize::MAX);
        let mut score_quality_generator = ScoreQuality::generate(pv_extension_counter);
        let score_quality = score_quality_generator.next();
        Self {
            game: None,
            pv_extension_counter,
            score_quality_generator,
            score_quality,
            node_count: 0,
            search_findings: AllSearchFindings::new(),
            persistent: Arc::new(Mutex::new(AlphaBetaPersistent::new())),
        }
    }

    fn set_game(&mut self, game: Game<G>) {
        self.score_quality_generator = ScoreQuality::generate(self.pv_extension_counter);
        self.score_quality = self.score_quality_generator.next();
        self.node_count = 0;
        self.search_findings = AllSearchFindings::new();
        self.game = Some(game);
    }

    fn think(&mut self, max_time: chrono::TimeDelta) {
        if let Some(game) = &self.game {
            let stop = chrono::Utc::now() + max_time;
            let mut state = game.state().clone();
            while !stop.stop() {
                if let Some(score_quality) = self.score_quality {
                    let mut node_count = 0;
                    if let Ok((score, best_move_at_depth)) =
                        negamax_alphabeta_score::<chrono::DateTime<chrono::Utc>, _>(
                            stop,
                            0,
                            game.logic(),
                            &mut state,
                            self.persistent.clone(),
                            score_quality,
                            0,
                            &mut node_count,
                            WithNegInf::NegInf,
                            WithPosInf::PosInf,
                        )
                    {
                        let current_best = &mut self.search_findings;
                        self.node_count += node_count;
                        if let Some(best_move) = best_move_at_depth {
                            current_best.update(SearchFindings {
                                score_quality,
                                score,
                                best_move,
                            });
                        }
                        self.score_quality = self.score_quality_generator.next();
                    }
                }
            }
        }
    }

    fn best_moves(&self) -> Vec<(String, G::Move)> {
        self.search_findings.best_moves()
    }
}
