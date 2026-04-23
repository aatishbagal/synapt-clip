//! Levenshtein edit distance and a fuzzy searcher over clip contents.
//!
//! Maps to the "dictionaries allowing errors" topic in Unit 3. Uses the
//! classic DP recurrence with the two-row space optimisation.

/// Edit distance between `a` and `b`, char-aware (safe for Unicode).
/// O(m * n) time, O(min(m, n)) space via two rolling rows.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.trim().to_lowercase().chars().collect();
    let b_chars: Vec<char> = b.trim().to_lowercase().chars().collect();

    let (short, long) = if a_chars.len() <= b_chars.len() {
        (&a_chars, &b_chars)
    } else {
        (&b_chars, &a_chars)
    };

    if short.is_empty() {
        return long.len();
    }

    let m = short.len();
    let n = long.len();

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for j in 1..=n {
        curr[0] = j;
        for i in 1..=m {
            let cost = if short[i - 1] == long[j - 1] { 0 } else { 1 };
            curr[i] = (curr[i - 1] + 1)
                .min(prev[i] + 1)
                .min(prev[i - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[m]
}

/// Return true when edit distance between `a` and `b` is at most `threshold`.
/// Early-exits when the length difference alone exceeds the threshold.
pub fn within_distance(a: &str, b: &str, threshold: usize) -> bool {
    let a_len = a.trim().chars().count();
    let b_len = b.trim().chars().count();
    let diff = a_len.abs_diff(b_len);
    if diff > threshold {
        return false;
    }
    levenshtein(a, b) <= threshold
}

/// Fuzzy searcher that matches a query against individual words inside
/// each clip's content.
pub struct FuzzySearcher {
    threshold: usize,
}

impl FuzzySearcher {
    /// Create a searcher with the given max edit distance. Typical default: 2.
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }

    /// Search `clips` for matches within the configured edit distance.
    /// Returns `(clip_id, min_distance)` sorted ascending.
    pub fn search(&self, query: &str, clips: &[(i64, String)]) -> Vec<(i64, usize)> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let q_len = q.chars().count();

        let mut out: Vec<(i64, usize)> = Vec::new();
        for (id, content) in clips {
            let mut best: Option<usize> = None;
            for word in content.split_whitespace() {
                let w_len = word.chars().count();
                if w_len.abs_diff(q_len) > self.threshold {
                    continue;
                }
                let d = levenshtein(&q, word);
                if d <= self.threshold {
                    best = Some(best.map_or(d, |b| b.min(d)));
                    if best == Some(0) {
                        break;
                    }
                }
            }
            if let Some(d) = best {
                out.push((*id, d));
            }
        }

        out.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        out
    }
}

impl Default for FuzzySearcher {
    fn default() -> Self {
        Self::new(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn identical_strings() {
        assert_eq!(levenshtein("abc", "abc"), 0);
    }

    #[test]
    fn single_deletion() {
        assert_eq!(levenshtein("abc", "ab"), 1);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(levenshtein("abc", "axc"), 1);
    }

    #[test]
    fn two_insertions() {
        assert_eq!(levenshtein("abc", "aXYc"), 2);
    }

    #[test]
    fn kitten_sitting() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn within_distance_early_exits_on_length() {
        assert!(!within_distance("a", "abcdef", 2));
        assert!(within_distance("abc", "abcd", 1));
    }

    #[test]
    fn fuzzy_finds_typo() {
        let searcher = FuzzySearcher::new(2);
        let clips = vec![(1, "please receive the letter".to_string())];
        let results = searcher.search("recieve", &clips);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
        assert!(results[0].1 <= 2);
    }

    #[test]
    fn fuzzy_skips_beyond_threshold() {
        let searcher = FuzzySearcher::new(1);
        let clips = vec![(1, "completely different".to_string())];
        let results = searcher.search("xyz", &clips);
        assert!(results.is_empty());
    }

    #[test]
    fn fuzzy_results_sorted_ascending() {
        let searcher = FuzzySearcher::new(3);
        let clips = vec![
            (1, "abcd".to_string()),  // distance 1 from "abc"
            (2, "abc".to_string()),   // distance 0
            (3, "axcd".to_string()),  // distance 2
        ];
        let results = searcher.search("abc", &clips);
        let distances: Vec<usize> = results.iter().map(|(_, d)| *d).collect();
        let mut sorted = distances.clone();
        sorted.sort();
        assert_eq!(distances, sorted);
        assert_eq!(results[0].0, 2);
    }

    #[test]
    fn unicode_distance() {
        // Substituting a multi-byte char should count as exactly one edit.
        assert_eq!(levenshtein("café", "cafe"), 1);
        assert_eq!(levenshtein("日本語", "日本"), 1);
    }
}
