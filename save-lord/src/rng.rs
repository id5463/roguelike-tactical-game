use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use serde::{Serialize, Deserialize};

/// Seeded RNG wrapper for deterministic gameplay.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeededRng {
    seed: u64,
    #[serde(skip, default = "default_rng")]
    rng: ChaCha20Rng,
}

fn default_rng() -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(0)
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha20Rng::seed_from_u64(seed),
        }
    }

    pub fn from_entropy() -> Self {
        let seed = rand::thread_rng().gen::<u64>();
        Self::new(seed)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn reseed(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = ChaCha20Rng::seed_from_u64(seed);
    }

    pub fn gen_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max { return min; }
        self.rng.gen_range(min..=max)
    }

    pub fn gen_range_f64(&mut self, min: f64, max: f64) -> f64 {
        if min >= max { return min; }
        self.rng.gen_range(min..=max)
    }

    pub fn gen_bool(&mut self, p: f64) -> bool {
        self.rng.gen_bool(p)
    }

    pub fn gen_percent(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }

    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() { return None; }
        let idx = self.rng.gen_range(0..slice.len());
        Some(&slice[idx])
    }

    pub fn choose_mut<'a, T>(&mut self, slice: &'a mut [T]) -> Option<&'a mut T> {
        if slice.is_empty() { return None; }
        let idx = self.rng.gen_range(0..slice.len());
        Some(&mut slice[idx])
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::prelude::SliceRandom;
        slice.shuffle(&mut self.rng);
    }
}
