pub struct BloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    item_count: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let n = expected_items.max(1) as f64;
        let p = false_positive_rate.clamp(1e-9, 0.999);
        let ln2 = std::f64::consts::LN_2;
        let num_bits = (-n * p.ln() / (ln2 * ln2)).ceil() as usize;
        let num_bits = num_bits.max(8);
        let num_hashes = ((num_bits as f64 / n) * ln2).round() as usize;
        let num_hashes = num_hashes.max(1);

        let words = num_bits.div_ceil(64);
        Self {
            bits: vec![0; words],
            num_bits,
            num_hashes,
            item_count: 0,
        }
    }

    pub fn insert(&mut self, item: &str) {
        let h1 = fnv1a(item);
        let h2 = djb2(item);
        for i in 0..self.num_hashes {
            let idx = self.hash_index(h1, h2, i);
            let word = idx / 64;
            let bit = idx % 64;
            self.bits[word] |= 1u64 << bit;
        }
        self.item_count += 1;
    }

    pub fn probably_contains(&self, item: &str) -> bool {
        let h1 = fnv1a(item);
        let h2 = djb2(item);
        for i in 0..self.num_hashes {
            let idx = self.hash_index(h1, h2, i);
            let word = idx / 64;
            let bit = idx % 64;
            if (self.bits[word] >> bit) & 1 == 0 {
                return false;
            }
        }
        true
    }

    pub fn false_positive_rate(&self) -> f64 {
        let exponent =
            -(self.num_hashes as f64) * (self.item_count as f64) / (self.num_bits as f64);
        (1.0 - exponent.exp()).powi(self.num_hashes as i32)
    }

    pub fn clear(&mut self) {
        for w in self.bits.iter_mut() {
            *w = 0;
        }
        self.item_count = 0;
    }

    fn hash_index(&self, h1: u64, h2: u64, i: usize) -> usize {
        let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
        (combined % self.num_bits as u64) as usize
    }
}

fn fnv1a(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn djb2(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.as_bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(*b as u64);
    }
    if hash == 0 {
        hash = 1;
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_probably_contains() {
        let mut b = BloomFilter::new(100, 0.01);
        b.insert("hello");
        assert!(b.probably_contains("hello"));
    }

    #[test]
    fn never_inserted_returns_false() {
        let mut b = BloomFilter::new(1000, 0.001);
        b.insert("only-one-item");
        assert!(!b.probably_contains("definitely-not-inserted-42"));
        assert!(!b.probably_contains("another-missing-xyz-789"));
    }

    #[test]
    fn false_positive_rate_stays_low() {
        let mut b = BloomFilter::new(1000, 0.01);
        for i in 0..100 {
            b.insert(&format!("item-{i}"));
        }
        assert!(b.false_positive_rate() < 0.05);
    }

    #[test]
    fn clear_resets_filter() {
        let mut b = BloomFilter::new(100, 0.01);
        b.insert("foo");
        assert!(b.probably_contains("foo"));
        b.clear();
        assert!(!b.probably_contains("foo"));
    }

    #[test]
    fn distinct_strings_do_not_collide() {
        let b_check = BloomFilter::new(1000, 0.001);
        let mut b = BloomFilter::new(1000, 0.001);
        b.insert("alpha");
        assert!(b.probably_contains("alpha"));
        assert!(!b.probably_contains("beta"));
        assert!(!b.probably_contains("gamma"));
        assert!(!b_check.probably_contains("alpha"));
    }
}
