use rand::rngs::StdRng;
use rand::{Rng as _, SeedableRng};

/// Thin wrapper so the sim never touches a global RNG. Deterministic given a seed.
pub struct Rng {
    pub seed: u64,
    inner: StdRng,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            inner: StdRng::seed_from_u64(seed),
        }
    }

    pub fn random(&mut self) -> f64 {
        self.inner.gen::<f64>()
    }

    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        if a == b {
            a
        } else {
            self.inner.gen_range(a..=b)
        }
    }

    /// Inclusive on both ends, matching Python `random.randint`.
    pub fn randint(&mut self, a: i32, b: i32) -> i32 {
        self.inner.gen_range(a..=b)
    }

    pub fn choice<'a, T>(&mut self, seq: &'a [T]) -> &'a T {
        let i = self.inner.gen_range(0..seq.len());
        &seq[i]
    }

    pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        // Polar Box–Muller.
        loop {
            let u = self.random() * 2.0 - 1.0;
            let v = self.random() * 2.0 - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let mul = (-2.0 * s.ln() / s).sqrt();
                return mu + sigma * u * mul;
            }
        }
    }

    pub fn weighted<T: Clone>(&mut self, items: &[(T, f64)]) -> T {
        let total: f64 = items.iter().map(|(_, w)| w).sum();
        let pick = self.random() * total;
        let mut acc = 0.0;
        for (item, w) in items {
            acc += w;
            if pick <= acc {
                return item.clone();
            }
        }
        items.last().unwrap().0.clone()
    }
}
