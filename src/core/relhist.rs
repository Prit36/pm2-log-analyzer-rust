const RELATIVE_ACCURACY: f64 = 0.01;
const GAMMA: f64 = (1.0 + RELATIVE_ACCURACY) / (1.0 - RELATIVE_ACCURACY);

const BUCKET_OFFSET: i32 = 400;
const NUM_BUCKETS: usize = 1200;
const INLINE_CAP: usize = 16;

#[derive(Clone, Debug, PartialEq)]
pub struct RelHist {
    pub keys: [u16; INLINE_CAP],
    pub counts: [u32; INLINE_CAP],
    pub len: u8,
    pub count: u32,
    pub overflow: Option<Box<Vec<(u16, u32)>>>,
}

impl Default for RelHist {
    fn default() -> Self {
        Self {
            keys: [0; INLINE_CAP],
            counts: [0; INLINE_CAP],
            len: 0,
            count: 0,
            overflow: None,
        }
    }
}

const LUT_LOG2: [f32; 256] = {
    let mut table = [0.0f32; 256];
    let mut i = 0;
    while i < 256 {
        let x = (i as f32 + 0.5) / 256.0;
        table[i] = x * (1.44269504 - 0.44269504 * x);
        i += 1;
    }
    table
};

#[inline(always)]
pub fn fast_log_bucket(value: f32) -> i32 {
    let bits = value.to_bits();
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant_idx = ((bits >> 15) & 0xFF) as usize;
    let log2_val = exp as f32 + LUT_LOG2[mant_idx];
    (log2_val * 34.657359027997266).ceil() as i32
}

impl RelHist {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn accept(&mut self, value: f32) {
        if !(value > 0.0) || !value.is_finite() {
            return;
        }
        let key = fast_log_bucket(value);
        let idx = (key + BUCKET_OFFSET).clamp(0, (NUM_BUCKETS - 1) as i32) as u16;
        self.add_count(idx, 1);
    }

    #[inline(always)]
    fn add_count(&mut self, idx: u16, cnt: u32) {
        self.count += cnt;
        let len = self.len as usize;
        for i in 0..len {
            if unsafe { *self.keys.get_unchecked(i) } == idx {
                unsafe { *self.counts.get_unchecked_mut(i) += cnt; }
                return;
            }
        }
        if len < INLINE_CAP {
            unsafe {
                *self.keys.get_unchecked_mut(len) = idx;
                *self.counts.get_unchecked_mut(len) = cnt;
            }
            self.len += 1;
            return;
        }
        let overflow = self.overflow.get_or_insert_with(|| Box::new(Vec::new()));
        for entry in overflow.iter_mut() {
            if entry.0 == idx {
                entry.1 += cnt;
                return;
            }
        }
        overflow.push((idx, cnt));
    }

    pub fn merge(&mut self, other: &RelHist) {
        if other.count == 0 {
            return;
        }
        let other_len = other.len as usize;
        for i in 0..other_len {
            self.add_count(other.keys[i], other.counts[i]);
        }
        if let Some(ref oflow) = other.overflow {
            for &(k, c) in oflow.iter() {
                self.add_count(k, c);
            }
        }
    }

    pub fn quantile(&self, q: f64) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let mut pairs: Vec<(u16, u32)> = Vec::with_capacity(self.len as usize + 8);
        for i in 0..self.len as usize {
            pairs.push((self.keys[i], self.counts[i]));
        }
        if let Some(ref oflow) = self.overflow {
            for &(k, c) in oflow.iter() {
                pairs.push((k, c));
            }
        }
        if pairs.is_empty() {
            return 0.0;
        }
        pairs.sort_unstable_by_key(|p| p.0);

        if q <= 0.0 {
            return bucket_value(pairs[0].0 as i32 - BUCKET_OFFSET);
        }
        if q >= 1.0 {
            return bucket_value(pairs.last().unwrap().0 as i32 - BUCKET_OFFSET);
        }
        let target = q * (self.count as f64 - 1.0);
        let mut rank = 0u32;
        for &(k, c) in &pairs {
            if (rank + c) as f64 > target {
                return bucket_value(k as i32 - BUCKET_OFFSET);
            }
            rank += c;
        }
        bucket_value(key_from_last(&pairs))
    }
}

fn key_from_last(pairs: &[(u16, u32)]) -> i32 {
    pairs.last().map(|p| p.0 as i32 - BUCKET_OFFSET).unwrap_or(0)
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
