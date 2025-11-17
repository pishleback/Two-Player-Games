use crate::game::{RelScore, State, StateIdent, WithNegInf, WithPosInf};
use crate::{
    ai::Ai,
    game::{Game, GameLogic},
};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
pub mod multithreaded;
pub mod singlethreaded;

#[derive(Debug, PartialEq, Eq)]
enum TranspositionTableEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PvExtensionCounter {
    extend_after: usize,
    reset_to: usize,
}

impl PartialOrd for PvExtensionCounter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PvExtensionCounter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.reset_to.cmp(&other.reset_to) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => self.extend_after.cmp(&other.extend_after).reverse(),
            Ordering::Greater => Ordering::Less,
        }
    }
}

impl PvExtensionCounter {
    pub fn new(extend_after: usize, reset_to: usize) -> Self {
        Self {
            extend_after,
            reset_to,
        }
    }

    pub fn increment(mut self) -> Self {
        if self.extend_after == 0 {
            self.extend_after = self.reset_to;
        } else {
            self.extend_after -= 1;
        }
        self
    }

    pub fn do_extension(&self) -> bool {
        self.extend_after == 0
    }
}

const MAX_QUIESCENCE_DEPTH: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreQuality {
    depth: usize,
    quiescence_depth: usize,
    pv_extension_counter: PvExtensionCounter,
}

impl ScoreQuality {
    pub fn quiescence_depth(&self) -> usize {
        self.depth + self.quiescence_depth
    }

    pub fn pv_depth(&self) -> usize {
        self.depth + {
            let mut t = 0;
            let mut pvec = self.pv_extension_counter;
            for _ in 0..self.depth {
                if pvec.do_extension() {
                    t += 1;
                }
                pvec = pvec.increment();
            }
            t
        }
    }
}

fn combine_cmp(a: Option<Ordering>, b: Option<Ordering>) -> Option<Ordering> {
    if let Some(a) = a
        && let Some(b) = b
    {
        match (a, b) {
            (Ordering::Less, Ordering::Less) => Some(Ordering::Less),
            (Ordering::Less, Ordering::Equal) => Some(Ordering::Less),
            (Ordering::Less, Ordering::Greater) => None,
            (Ordering::Equal, Ordering::Less) => Some(Ordering::Less),
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            (Ordering::Equal, Ordering::Greater) => Some(Ordering::Greater),
            (Ordering::Greater, Ordering::Less) => None,
            (Ordering::Greater, Ordering::Equal) => Some(Ordering::Greater),
            (Ordering::Greater, Ordering::Greater) => Some(Ordering::Greater),
        }
    } else {
        None
    }
}

impl PartialOrd for ScoreQuality {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        combine_cmp(
            combine_cmp(
                Some(self.depth.cmp(&other.depth)),
                Some(
                    (self.depth + self.quiescence_depth)
                        .cmp(&(other.depth + other.quiescence_depth)),
                ),
            ),
            Some(self.pv_extension_counter.cmp(&other.pv_extension_counter)),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SubjectiveScoreQuality {
    q: ScoreQuality,
}

impl PartialOrd for SubjectiveScoreQuality {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SubjectiveScoreQuality {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(ord) = self.q.partial_cmp(&other.q) {
            ord
        } else {
            let cmp = self.q.depth.cmp(&other.q.depth);
            if !cmp.is_eq() {
                return cmp;
            }
            let cmp = self.q.quiescence_depth().cmp(&other.q.quiescence_depth());
            if !cmp.is_eq() {
                return cmp;
            }
            let cmp = self.q.pv_depth().cmp(&other.q.pv_depth());
            if !cmp.is_eq() {
                return cmp;
            }
            Ordering::Equal
        }
    }
}

#[derive(Debug)]
pub struct ScoreQualityGenerator {
    depth: usize,
    quiescence_depth: usize,
    pv_extension_counter: PvExtensionCounter,
}

impl Iterator for ScoreQualityGenerator {
    type Item = ScoreQuality;

    fn next(&mut self) -> Option<Self::Item> {
        let next = ScoreQuality {
            depth: self.depth,
            quiescence_depth: self.quiescence_depth,
            pv_extension_counter: self.pv_extension_counter,
        };
        if self.quiescence_depth < MAX_QUIESCENCE_DEPTH {
            if self.quiescence_depth < 100 {
                self.quiescence_depth *= 2;
            }

            if self.quiescence_depth > MAX_QUIESCENCE_DEPTH {
                self.quiescence_depth = MAX_QUIESCENCE_DEPTH
            }
        } else {
            if self.depth > 100 {
                return None;
            }
            self.depth += 1;
        }

        Some(next)
    }
}

impl ScoreQuality {
    fn generate(pv_extension_counter: PvExtensionCounter) -> ScoreQualityGenerator {
        ScoreQualityGenerator {
            depth: 1,
            quiescence_depth: 1,
            pv_extension_counter,
        }
    }

    fn decrement(self) -> Option<Self> {
        if self.depth > 0 {
            Some(Self {
                depth: self.depth - 1,
                quiescence_depth: self.quiescence_depth,
                pv_extension_counter: self.pv_extension_counter.increment(),
            })
        } else if self.quiescence_depth > 0 {
            Some(Self {
                depth: self.depth,
                quiescence_depth: self.quiescence_depth - 1,
                pv_extension_counter: self.pv_extension_counter.increment(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct TranspositionTableEntry<G: GameLogic + Send> {
    score_quality: ScoreQuality,
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
                #[cfg(false)]
                {
                    // For debugging bad hashes
                    pub fn print_debug_diff_count<T: std::fmt::Debug, U: std::fmt::Debug>(
                        a: &T,
                        b: &U,
                    ) -> usize {
                        let s1 = format!("{:#?}", a);
                        let s2 = format!("{:#?}", b);

                        let lines1: Vec<&str> = s1.lines().collect();
                        let lines2: Vec<&str> = s2.lines().collect();

                        // Iterate over the maximum number of lines in either string
                        let max_len = lines1.len().max(lines2.len());

                        let mut diff_count = 0;

                        for i in 0..max_len {
                            let l1 = lines1.get(i).copied().unwrap_or("");
                            let l2 = lines2.get(i).copied().unwrap_or("");
                            if l1 != l2 {
                                println!("{}    !=    {}", l1, l2);
                                diff_count += 1;
                            }
                        }

                        diff_count
                    }

                    println!("Diff");
                    println!("{}", print_debug_diff_count(&entry.state, &state));
                }

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
    score_quality: ScoreQuality,
    depth_from_root: usize,
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
        && tt_entry.score_quality >= score_quality
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
    let (moves, mut best_score) = if score_quality.depth == 0 {
        let stand_pat = logic.score(state).into_rel(player);
        let stand_pat_with_neg_inf = WithNegInf::Finite(stand_pat.clone());
        if alpha < stand_pat_with_neg_inf {
            alpha = stand_pat_with_neg_inf.clone();
        }
        if alpha >= beta {
            return Ok((stand_pat, None));
        }
        if score_quality.quiescence_depth == 0 {
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
    let mut best_move_idx = None;
    'SEARCH: {
        let n = ordered_moves.len();
        let mut ordered_scores = vec![];
        for (idx, mv) in ordered_moves.iter().enumerate() {
            #[cfg(debug_assertions)]
            let state_before = (*state).clone();
            logic.make_move(state, mv);
            debug_assert_ne!(logic.turn(state), player);
            let (score, _) = negamax_alphabeta_score::<S, G>(
                stop.clone(),
                thread_num,
                logic,
                state,
                persistent.clone(),
                score_quality.decrement().unwrap(),
                depth_from_root + 1,
                node_count,
                -beta.clone().map(|v| v.dec_time()),
                -alpha.clone().map(|v| v.dec_time()),
            )?;
            let score = -score;
            let score = score.inc_time();
            logic.unmake_move(state, mv);
            #[cfg(debug_assertions)]
            assert_eq!(*state, state_before);

            let score = WithNegInf::Finite(score);
            if best_score < score {
                best_score = score.clone();
                best_move_idx = Some(idx);
            }
            if alpha < score {
                alpha = score.clone();
            }
            ordered_scores.push(score);
            if alpha >= beta {
                break 'SEARCH;
            }
        }

        debug_assert_eq!(n, ordered_moves.len());
        debug_assert_eq!(n, ordered_scores.len());

        // PV extensions
        if let Some(mut best_move_idx) = best_move_idx
            && score_quality.pv_extension_counter.do_extension()
        {
            let mut ordered_extended_scores = vec![None; n];
            let score_quality = {
                let mut score_quality = score_quality;
                score_quality.pv_extension_counter = score_quality.pv_extension_counter.increment();
                score_quality
            };

            loop {
                if ordered_extended_scores[best_move_idx].is_some() {
                    // we've already done an extended search here
                    break 'SEARCH;
                }

                logic.make_move(state, &ordered_moves[best_move_idx]);
                debug_assert_ne!(logic.turn(state), player);
                let (score, _) = negamax_alphabeta_score::<S, G>(
                    stop.clone(),
                    thread_num,
                    logic,
                    state,
                    persistent.clone(),
                    score_quality,
                    depth_from_root + 1,
                    node_count,
                    -beta.clone().map(|v| v.dec_time()),
                    -alpha.clone().map(|v| v.dec_time()),
                )?;
                let score = -score;
                let score = score.inc_time();
                let score = WithNegInf::Finite(score);
                logic.unmake_move(state, &ordered_moves[best_move_idx]);

                ordered_extended_scores[best_move_idx] = Some(score.clone());

                if score < best_score {
                    let new_best_move_idx = (0..n)
                        .max_by_key(|i| {
                            ordered_extended_scores[*i]
                                .as_ref()
                                .unwrap_or(&ordered_scores[*i])
                        })
                        .unwrap();
                    best_score = ordered_extended_scores[new_best_move_idx]
                        .as_ref()
                        .unwrap_or(&ordered_scores[new_best_move_idx])
                        .clone();
                    if new_best_move_idx == best_move_idx {
                        break 'SEARCH;
                    }
                    best_move_idx = new_best_move_idx
                } else {
                    break 'SEARCH;
                }
            }
        }
    }
    let best_move = best_move_idx.map(|idx| ordered_moves[idx].clone());

    if depth_from_root == 2 {
        state.set_ignore_repetitions(false);
    }

    // Transposition Table store

    let mut persistent = persistent.lock().unwrap();

    let tt_entry_opt = persistent.transpositions.get(state.clone().ident());
    if tt_entry_opt
        .as_ref()
        .map(|tt_entry| tt_entry.score_quality < score_quality)
        .unwrap_or(true)
    {
        *tt_entry_opt = Some(TranspositionTableEntry {
            score_quality,
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
    }

    Ok((best_score.unwrap_finite(), best_move))
}

#[derive(Debug)]
struct SearchFindings<G: GameLogic> {
    score_quality: ScoreQuality,
    score: RelScore<G::HeuristicScore>,
    best_move: G::Move,
}

#[derive(Debug)]
struct AllSearchFindings<G: GameLogic> {
    all_findings: Vec<SearchFindings<G>>,
}

impl<G: GameLogic> AllSearchFindings<G> {
    pub fn new() -> Self {
        Self {
            all_findings: vec![],
        }
    }

    pub fn best_moves(&self) -> Vec<(String, G::Move)> {
        self.all_findings
            .iter()
            .map(|finding| {
                (
                    format!(
                        "D={} E={} {}/{}{} S={}",
                        finding.score_quality.depth,
                        finding.score_quality.pv_depth(),
                        if finding.score_quality.pv_extension_counter.extend_after < usize::MAX / 2
                        {
                            format!(
                                "{}",
                                finding.score_quality.pv_extension_counter.extend_after
                            )
                        } else {
                            "∞".to_string()
                        },
                        if finding.score_quality.pv_extension_counter.reset_to < usize::MAX / 2 {
                            format!("{}", finding.score_quality.pv_extension_counter.reset_to)
                        } else {
                            "∞".to_string()
                        },
                        if finding.score_quality.quiescence_depth == MAX_QUIESCENCE_DEPTH {
                            "".to_string()
                        } else {
                            format!(" Q={}", finding.score_quality.quiescence_depth())
                        },
                        match &finding.score {
                            RelScore::Heuristic(score) => format!("{:?}", score),
                            RelScore::Terminal(terminal, time) => {
                                match terminal {
                                    crate::game::RelTerminal::Lose => format!("Lose({time})"),
                                    crate::game::RelTerminal::Draw => format!("Draw({time})"),
                                    crate::game::RelTerminal::Win => format!("Win({time})"),
                                }
                            }
                        }
                    ),
                    finding.best_move.clone(),
                )
            })
            .collect()
    }

    pub fn update(&mut self, new_findings: SearchFindings<G>) {
        self.all_findings.push(new_findings);
        'LOOP: loop {
            let n = self.all_findings.len();
            for i in 0..n {
                if (0..n).filter(|j| i != *j).any(|j| {
                    self.all_findings[j].score_quality >= self.all_findings[i].score_quality
                }) {
                    self.all_findings.remove(i);
                    continue 'LOOP;
                }
            }
            break;
        }
        self.all_findings
            .sort_by_cached_key(|f| SubjectiveScoreQuality { q: f.score_quality });
        self.all_findings.reverse();
    }
}
