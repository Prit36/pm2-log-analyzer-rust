use std::collections::HashMap;

const RELATIVE_ACCURACY: f64 = 0.01;
const GAMMA: f64 = (1.0 + RELATIVE_ACCURACY) / (1.0 - RELATIVE_ACCURACY);
/// 1/ln(γ); precomputed so accept() never calls ln(γ)
const INV_LOG_GAMMA: f64 = 1.0 / 0.020000666688891502;

#[derive(Clone, Debug)]
pub struct RelHist {
    buckets: HashMap<i32, u32>,
    pub count: u32,
}

impl Default for RelHist {
    fn default() -> Self {
        Self {
            buckets: HashMap::new(),
            count: 0,
        }
    }
}

impl RelHist {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn accept(&mut self, value: f32) {
        let v = value as f64;
        if !(v > 0.0) || !v.is_finite() {
            return;
        }
        let key = (v.ln() * INV_LOG_GAMMA).ceil() as i32;
        *self.buckets.entry(key).or_insert(0) += 1;
        self.count += 1;
    }

    pub fn merge(&mut self, other: &RelHist) {
        for (&k, &c) in &other.buckets {
            *self.buckets.entry(k).or_insert(0) += c;
        }
        self.count += other.count;
    }

    pub fn quantile(&self, q: f64) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let mut keys: Vec<i32> = self.buckets.keys().copied().collect();
        keys.sort_unstable();
        if q <= 0.0 {
            return bucket_value(keys[0]);
        }
        if q >= 1.0 {
            return bucket_value(*keys.last().unwrap());
        }
        let target = q * (self.count as f64 - 1.0);
        let mut rank = 0u32;
        for &k in &keys {
            let c = *self.buckets.get(&k).unwrap();
            if (rank + c) as f64 > target {
                return bucket_value(k);
            }
            rank += c;
        }
        bucket_value(*keys.last().unwrap())
    }
}

fn bucket_value(key: i32) -> f32 {
    GAMMA.powf(key as f64 - 0.5) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantile_approx() {
        let mut h = RelHist::new();
        for i in 1..=1000 {
            h.accept((i * 10) as f32);
        }
        let p95 = h.quantile(0.95);
        assert!((p95 - 9500.0).abs() / 9500.0 < 0.03, "p95={p95}");
    }
}
