use hashbrown::HashMap;

#[allow(dead_code)]
const RELATIVE_ACCURACY: f64 = 0.01;
#[allow(dead_code)]
const GAMMA: f64 = (1.0 + RELATIVE_ACCURACY) / (1.0 - RELATIVE_ACCURACY);
/// 1/ln(γ); precomputed so accept() never calls ln(γ) (value.ln() still per sample).
pub const INV_LOG_GAMMA: f64 = 1.0 / 0.020000666688891502; // == 1.0 / GAMMA.ln()
pub const DENSE_LIMIT: usize = 512;

#[inline(always)]
pub fn relhist_key(value: f32) -> Option<i32> {
    let v = value as f64;
    if !(v > 0.0) || !v.is_finite() {
        None
    } else {
        Some((v.ln() * INV_LOG_GAMMA).ceil() as i32)
    }
}

/// Bucket value used for quantiles; matches `core::relhist_js::bucket_value`
/// (f32 table for dense keys, f64 `powf` outside it).
#[inline]
fn bucket_value(key: i32) -> f64 {
    if key >= 0 && (key as usize) < DENSE_LIMIT {
        (GAMMA.powf(key as f64 - 0.5) as f32) as f64
    } else {
        GAMMA.powf(key as f64 - 0.5)
    }
}

#[derive(Clone, Debug)]
pub struct RelHist {
    dense: [u32; DENSE_LIMIT],
    sparse: HashMap<i32, u32>,
    pub count: u32,
}

impl Default for RelHist {
    fn default() -> Self {
        Self {
            dense: [0; DENSE_LIMIT],
            sparse: HashMap::new(),
            count: 0,
        }
    }
}

impl RelHist {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn accept_key(&mut self, key: i32) {
        self.count += 1;
        if key >= 0 && (key as usize) < DENSE_LIMIT {
            self.dense[key as usize] += 1;
        } else {
            *self.sparse.entry(key).or_insert(0) += 1;
        }
    }

    /// Add another sketch's bucket counts (native coordinator merge).
    pub fn merge(&mut self, other: &RelHist) {
        self.count += other.count;
        for i in 0..DENSE_LIMIT {
            self.dense[i] += other.dense[i];
        }
        for (&key, &count) in &other.sparse {
            *self.sparse.entry(key).or_insert(0) += count;
        }
    }

    /// Iterate buckets in ascending key order (same order as `to_wire`).
    fn for_each_bucket(&self, mut f: impl FnMut(i32, u64)) {
        let mut neg: Vec<i32> = self.sparse.keys().copied().filter(|&k| k < 0).collect();
        neg.sort_unstable();
        for key in neg {
            f(key, self.sparse[&key] as u64);
        }
        for (key, &count) in self.dense.iter().enumerate() {
            if count > 0 {
                f(key as i32, count as u64);
            }
        }
        let mut high: Vec<i32> = self
            .sparse
            .keys()
            .copied()
            .filter(|&k| k >= DENSE_LIMIT as i32)
            .collect();
        high.sort_unstable();
        for key in high {
            f(key, self.sparse[&key] as u64);
        }
    }

    /// JS-parity quantile (mirrors `relHist.quantile` in the reference coordinator).
    pub fn quantile(&self, q: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let mut first = None;
        let mut last = 0i32;
        self.for_each_bucket(|key, _| {
            if first.is_none() {
                first = Some(key);
            }
            last = key;
        });
        let first = first.unwrap_or(0);
        if q <= 0.0 {
            return bucket_value(first);
        }
        if q >= 1.0 {
            return bucket_value(last);
        }
        let target = q * (self.count as f64 - 1.0);
        let mut rank = 0u64;
        let mut found = None;
        self.for_each_bucket(|key, count| {
            if found.is_none() && (rank + count) as f64 > target {
                found = Some(key);
            }
            rank += count;
        });
        bucket_value(found.unwrap_or(last))
    }

    /// One pass for p50/p90/p95/p99 (mirrors `RelHist.quantiles4()`).
    pub fn quantiles4(&self) -> [f64; 4] {
        if self.count == 0 {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let count = self.count as f64;
        let t50 = 0.5 * (count - 1.0);
        let t90 = 0.9 * (count - 1.0);
        let t95 = 0.95 * (count - 1.0);
        let t99 = 0.99 * (count - 1.0);
        let mut p50 = f64::NAN;
        let mut p90 = f64::NAN;
        let mut p95 = f64::NAN;
        let mut p99 = f64::NAN;
        let mut rank = 0u64;
        let mut last = 0i32;
        self.for_each_bucket(|key, bucket| {
            last = key;
            let next_rank = rank + bucket;
            let value = bucket_value(key);
            if p50.is_nan() && next_rank as f64 > t50 {
                p50 = value;
            }
            if p90.is_nan() && next_rank as f64 > t90 {
                p90 = value;
            }
            if p95.is_nan() && next_rank as f64 > t95 {
                p95 = value;
            }
            if p99.is_nan() && next_rank as f64 > t99 {
                p99 = value;
            }
            rank = next_rank;
        });
        let last_value = bucket_value(last);
        [
            if p50.is_nan() { last_value } else { p50 },
            if p90.is_nan() { last_value } else { p90 },
            if p95.is_nan() { last_value } else { p95 },
            if p99.is_nan() { last_value } else { p99 },
        ]
    }

    /// Encode as [count:u32][n:u32][key:i32, cnt:u32]×n little-endian (keys sorted).
    pub fn to_wire(&self) -> Vec<u8> {
        let mut neg_keys: Vec<i32> = self.sparse.keys().copied().filter(|&k| k < 0).collect();
        neg_keys.sort_unstable();

        let mut high_keys: Vec<i32> = self
            .sparse
            .keys()
            .copied()
            .filter(|&k| k >= DENSE_LIMIT as i32)
            .collect();
        high_keys.sort_unstable();

        let mut dense_count = 0usize;
        for i in 0..DENSE_LIMIT {
            if self.dense[i] > 0 {
                dense_count += 1;
            }
        }

        let total_n = neg_keys.len() + dense_count + high_keys.len();
        let mut out = Vec::with_capacity(8 + total_n * 8);
        out.extend_from_slice(&self.count.to_le_bytes());
        out.extend_from_slice(&(total_n as u32).to_le_bytes());

        for k in neg_keys {
            let c = self.sparse[&k];
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&c.to_le_bytes());
        }
        for i in 0..DENSE_LIMIT {
            let c = self.dense[i];
            if c > 0 {
                let k = i as i32;
                out.extend_from_slice(&k.to_le_bytes());
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        for k in high_keys {
            let c = self.sparse[&k];
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&c.to_le_bytes());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket_value(key: i32) -> f32 {
        GAMMA.powf(key as f64 - 0.5) as f32
    }

    fn quantile(h: &RelHist, q: f64) -> f32 {
        if h.count == 0 {
            return 0.0;
        }
        let wire = h.to_wire();
        let n = u32::from_le_bytes(wire[4..8].try_into().unwrap()) as usize;
        let mut keys = Vec::with_capacity(n);
        let mut counts = Vec::with_capacity(n);
        let mut off = 8usize;
        for _ in 0..n {
            let k = i32::from_le_bytes(wire[off..off + 4].try_into().unwrap());
            let c = u32::from_le_bytes(wire[off + 4..off + 8].try_into().unwrap());
            keys.push(k);
            counts.push(c);
            off += 8;
        }
        if q <= 0.0 {
            return bucket_value(keys[0]);
        }
        if q >= 1.0 {
            return bucket_value(*keys.last().unwrap());
        }
        let target = q * (h.count as f64 - 1.0);
        let mut rank = 0u32;
        for i in 0..n {
            let c = counts[i];
            if (rank + c) as f64 > target {
                return bucket_value(keys[i]);
            }
            rank += c;
        }
        bucket_value(*keys.last().unwrap())
    }

    #[test]
    fn kernel_quantiles_match_js_coordinator() {
        use crate::core::relhist_js::{RelHist as JsRelHist, RelHistWire};
        let mut kernel = RelHist::new();
        let mut js = JsRelHist::new();
        for i in 0..20_000i32 {
            let key = match i % 5 {
                0 => (i % 400) - 30,
                1 => 512 + (i % 50),
                2 => -200 + (i % 100),
                3 => i % 511,
                _ => 700 + (i % 20),
            };
            kernel.accept_key(key);
            js.merge_wire(&RelHistWire {
                count: 1,
                buckets: vec![(key, 1)],
            });
        }
        assert_eq!(kernel.count as u64, js.count);
        for q in [0.0, 0.01, 0.25, 0.5, 0.9, 0.95, 0.99, 1.0] {
            assert_eq!(kernel.quantile(q), js.quantile(q), "q={q}");
        }
        assert_eq!(kernel.quantiles4(), js.quantiles4());
    }

    #[test]
    fn quantile_approx() {
        let mut h = RelHist::new();
        for i in 1..=1000 {
            h.accept_key(relhist_key((i * 10) as f32).unwrap());
        }
        let p95 = quantile(&h, 0.95);
        assert!((p95 - 9500.0).abs() / 9500.0 < 0.02, "p95={p95}");
    }
}
