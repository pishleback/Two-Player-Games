use crate::game::{Game, GameLogic};

pub mod alphabeta;
pub mod null;
pub mod random;

/// Fast bijective 64 -> 64 using a 3-round Feistel network on 32-bit halves.
///
/// - `hash64(x)` maps u64 -> u64 bijectively.
/// - `unhash64(y)` is the inverse (recovers original).
///
/// This is simple, fast, and invertible. If you need *stronger* mixing,
/// increase rounds (e.g. 5 or 7) or make the round function stronger.
pub fn hash64(v: u64) -> u64 {
    #[inline]
    fn round_f(x: u32, k: u32) -> u32 {
        // cheap non-linear mixing: multiply (odd constant), rotate, xor
        // keep operations on 32-bit halves for speed and locality.
        let mut v = x.wrapping_mul(k);
        v = v.rotate_left(13);
        v ^ (x >> 5)
    }

    #[inline]
    fn split64(x: u64) -> (u32, u32) {
        ((x >> 32) as u32, x as u32)
    }

    #[inline]
    fn join64(hi: u32, lo: u32) -> u64 {
        ((hi as u64) << 32) | (lo as u64)
    }

    // Random odd constants
    const K0: u32 = 0x9E3779B1u32;
    const K1: u32 = 0xC2B2AE35u32;
    const K2: u32 = 0x165667B1u32;

    let (mut l, mut r) = split64(v);

    // 3-round Feistel (L, R swapped each round as usual)
    // Round 0
    let t = round_f(r, K0);
    l ^= t;
    // Round 1
    let t = round_f(l, K1);
    r ^= t;
    // Round 2
    let t = round_f(r, K2);
    l ^= t;

    // after odd number of rounds, swap halves to follow Feistel convention
    join64(r, l)
}

pub trait Ai<G: GameLogic> {
    fn new() -> Self;
    fn set_game(&mut self, game: Game<G>);
    fn think(&mut self, max_time: chrono::TimeDelta);
    fn best_moves(&self) -> Vec<(String, G::Move)>;
}
