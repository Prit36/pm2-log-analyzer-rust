//! JS-parity relative-error sketch — port of `src/parser/relHist.ts`.
//!
//! Only the coordinator paths are implemented: merging kernel wire buckets and
//! computing quantiles. Key generation lives in the vendored kernel and must not
//! be duplicated here.

use std::collections::BTreeMap;
use std::sync::LazyLock;

const RELATIVE_ACCURACY: f64 = 0.01;
const GAMMA: f64 = (1.0 + RELATIVE_ACCURACY) / (1.0 - RELATIVE_ACCURACY);
const BUCKET_TABLE_SIZE: usize = 512;

static BUCKET_TABLE: LazyLock<[f32; BUCKET_TABLE_SIZE]> = LazyLock::new(|| {
    let mut table = [0.0f32; BUCKET_TABLE_SIZE];
    for (k, slot) in table.iter_mut().enumerate() {
        *slot = GAMMA.powf(k as f64 - 0.5) as f32;
    }
    table
});

fn bucket_value(key: i32) -> f64 {
    if key >= 0 && (key as usize) < BUCKET_TABLE_SIZE {
        BUCKET_TABLE[key as usize] as f64
    } else {
        GAMMA.powf(key as f64 - 0.5)
    }
}

#[derive(Clone, Debug, Default)]
pub struct RelHist {
    buckets: BTreeMap<i32, u64>,
    pub count: u64,
}

impl RelHist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_wire(&mut self, wire: &RelHistWire) {
        for &(k, c) in &wire.buckets {
            *self.buckets.entry(k).or_insert(0) += c as u64;
        }
        self.count += wire.count as u64;
    }

    /// Serialize back to wire form (keys sorted; order is irrelevant to consumers).
    pub fn to_wire(&self) -> RelHistWire {
        RelHistWire {
            count: self.count.min(u32::MAX as u64) as u32,
            buckets: self
                .buckets
                .iter()
                .map(|(&k, &c)| (k, c.min(u32::MAX as u64) as u32))
                .collect(),
        }
    }

    pub fn quantile(&self, q: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let keys: Vec<i32> = self.buckets.keys().copied().collect();
        let first = *keys.first().unwrap();
        let last = *keys.last().unwrap();
        if q <= 0.0 {
            return bucket_value(first);
        }
        if q >= 1.0 {
            return bucket_value(last);
        }
        let target = q * (self.count as f64 - 1.0);
        let mut rank = 0u64;
        for k in &keys {
            let c = self.buckets[k];
            if (rank + c) as f64 > target {
                return bucket_value(*k);
            }
            rank += c;
        }
        bucket_value(last)
    }

    /// One pass for p50/p90/p95/p99 — mirrors `RelHist.quantiles4()`.
    pub fn quantiles4(&self) -> [f64; 4] {
        let count = self.count;
        if count == 0 {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let keys: Vec<i32> = self.buckets.keys().copied().collect();
        let last_val = bucket_value(*keys.last().unwrap());
        let t50 = 0.5 * (count as f64 - 1.0);
        let t90 = 0.9 * (count as f64 - 1.0);
        let t95 = 0.95 * (count as f64 - 1.0);
        let t99 = 0.99 * (count as f64 - 1.0);
        let mut p50 = f64::NAN;
        let mut p90 = f64::NAN;
        let mut p95 = f64::NAN;
        let mut p99 = f64::NAN;
        let mut rank = 0u64;
        for k in &keys {
            let c = self.buckets[k];
            let next_rank = rank + c;
            let v = bucket_value(*k);
            if p50.is_nan() && next_rank as f64 > t50 {
                p50 = v;
            }
            if p90.is_nan() && next_rank as f64 > t90 {
                p90 = v;
            }
            if p95.is_nan() && next_rank as f64 > t95 {
                p95 = v;
            }
            if p99.is_nan() && next_rank as f64 > t99 {
                p99 = v;
            }
            rank = next_rank;
        }
        [
            if p50.is_nan() { last_val } else { p50 },
            if p90.is_nan() { last_val } else { p90 },
            if p95.is_nan() { last_val } else { p95 },
            if p99.is_nan() { last_val } else { p99 },
        ]
    }
}

/// Decoded form of the kernel sketch wire: `[count:u32][n:u32][key:i32, cnt:u32]×n`.
#[derive(Clone, Debug, Default)]
pub struct RelHistWire {
    pub count: u32,
    pub buckets: Vec<(i32, u32)>,
}

impl RelHistWire {
    pub fn decode(buf: &[u8], off: &mut usize) -> RelHistWire {
        let count = read_u32(buf, *off);
        let n = read_u32(buf, *off + 4) as usize;
        *off += 8;
        let mut buckets = Vec::with_capacity(n);
        for _ in 0..n {
            let k = read_i32(buf, *off);
            let c = read_u32(buf, *off + 4);
            buckets.push((k, c));
            *off += 8;
        }
        RelHistWire { count, buckets }
    }
}

pub fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
    ])
}

pub fn read_i32(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
    ])
}

pub fn read_f32(buf: &[u8], off: usize) -> f32 {
    f32::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
    ])
}

pub fn read_f64(buf: &[u8], off: usize) -> f64 {
    f64::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
        buf[off + 4],
        buf[off + 5],
        buf[off + 6],
        buf[off + 7],
    ])
}

pub fn read_u16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_match_expected_buckets() {
        // 1..=1000 ms samples land one per bucket; p95 must be within 2% of 950.
        let mut hist = RelHist::new();
        for i in 1..=1000i32 {
            let key = (f64::from(i).ln() * (1.0 / GAMMA.ln())).ceil() as i32;
            hist.merge_wire(&RelHistWire {
                count: 1,
                buckets: vec![(key, 1)],
            });
        }
        let [p50, _p90, p95, p99] = hist.quantiles4();
        assert!((p50 - 500.0).abs() / 500.0 < 0.02, "p50={p50}");
        assert!((p95 - 950.0).abs() / 950.0 < 0.02, "p95={p95}");
        assert!((p99 - 990.0).abs() / 990.0 < 0.02, "p99={p99}");
    }
}
