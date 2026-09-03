//! A tiny deterministic pseudo-random generator (xorshift64).
//!
//! What it is: just enough randomness for wind gusts and cloud heights, owned by
//! the [`crate::world::World`] and threaded explicitly through the tick. Because
//! it is seedable and self-contained, the whole simulation is reproducible — the
//! same seed and inputs always produce the same run, which makes it testable.
//!
//! What it is not: cryptographically strong, or a general-purpose RNG.

/// A seedable xorshift64 generator.
#[derive(Clone, Copy)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A fixed default seed, so `World::default()` runs are deterministic in tests.
    const DEFAULT_SEED: u64 = 0x2545_f491_4f6c_dd1d;

    #[must_use]
    pub fn new(seed: u64) -> Self {
        // xorshift can never leave the all-zero state, so avoid seeding into it.
        Self {
            state: if seed == 0 { Self::DEFAULT_SEED } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// A uniform integer in `[low, high)`, matching the half-open bounds of the
    /// original's `Random`-based `gen_range` calls.
    ///
    /// # Panics
    /// Panics if `low >= high` (an empty range).
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        assert!(low < high, "empty range");
        let span = (high - low) as u64;
        low + (self.next_u64() % span) as i32
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SEED)
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn range_stays_within_bounds() {
        let mut rng = Rng::new(1);
        for _ in 0..10_000 {
            let v = rng.range(-2, 3);
            assert!((-2..3).contains(&v));
        }
    }

    #[test]
    fn same_seed_reproduces_the_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.range(0, 1000), b.range(0, 1000));
        }
    }
}
