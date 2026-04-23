use std::collections::HashMap;

const MAX_INDEX_LEN: usize = 500;

pub struct SuffixArray {
    text: String,
    suffixes: Vec<usize>,
    chars: Vec<char>,
}

impl SuffixArray {
    pub fn build(text: &str) -> Self {
        let normalized = text.to_lowercase();
        let chars: Vec<char> = normalized.chars().collect();
        let mut suffixes: Vec<usize> = (0..chars.len()).collect();
        suffixes.sort_by(|a, b| chars[*a..].cmp(&chars[*b..]));
        Self {
            text: normalized,
            suffixes,
            chars,
        }
    }

    pub fn contains(&self, pattern: &str) -> bool {
        let pat: Vec<char> = pattern.to_lowercase().chars().collect();
        if pat.is_empty() {
            return false;
        }
        if pat.len() > self.chars.len() {
            return false;
        }

        let mut lo = 0usize;
        let mut hi = self.suffixes.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let suf_start = self.suffixes[mid];
            let end = (suf_start + pat.len()).min(self.chars.len());
            let suf_slice = &self.chars[suf_start..end];
            match suf_slice.cmp(pat.as_slice()) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return true,
            }
        }
        false
    }

    pub fn find_all(&self, pattern: &str) -> Vec<usize> {
        let pat: Vec<char> = pattern.to_lowercase().chars().collect();
        let mut out = Vec::new();
        if pat.is_empty() || pat.len() > self.chars.len() {
            return out;
        }

        let mut lo = 0usize;
        let mut hi = self.suffixes.len();
        let mut found: Option<usize> = None;
        while lo < hi {
            let mid = (lo + hi) / 2;
            let suf_start = self.suffixes[mid];
            let end = (suf_start + pat.len()).min(self.chars.len());
            let suf_slice = &self.chars[suf_start..end];
            match suf_slice.cmp(pat.as_slice()) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => {
                    found = Some(mid);
                    break;
                }
            }
        }

        let Some(center) = found else {
            return out;
        };

        let matches_at = |i: usize| -> bool {
            let s = self.suffixes[i];
            let end = (s + pat.len()).min(self.chars.len());
            self.chars[s..end] == pat[..]
        };

        let mut i = center;
        loop {
            if matches_at(i) {
                out.push(self.suffixes[i]);
            } else {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        let mut j = center + 1;
        while j < self.suffixes.len() && matches_at(j) {
            out.push(self.suffixes[j]);
            j += 1;
        }
        out.sort();
        out
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

pub struct SuffixIndex {
    entries: HashMap<i64, SuffixArray>,
}

impl SuffixIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn index_clip(&mut self, clip_id: i64, content: &str) {
        let truncated: String = content.chars().take(MAX_INDEX_LEN).collect();
        self.entries.insert(clip_id, SuffixArray::build(&truncated));
    }

    pub fn remove_clip(&mut self, clip_id: i64) {
        self.entries.remove(&clip_id);
    }

    pub fn search(&self, pattern: &str) -> Vec<i64> {
        let mut out = Vec::new();
        for (id, sa) in &self.entries {
            if sa.contains(pattern) && !out.contains(id) {
                out.push(*id);
            }
        }
        out
    }

    pub fn get(&self, clip_id: i64) -> Option<&SuffixArray> {
        self.entries.get(&clip_id)
    }
}

impl Default for SuffixIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_exact_match() {
        let sa = SuffixArray::build("hello");
        assert!(sa.contains("hello"));
    }

    #[test]
    fn contains_middle_substring() {
        let sa = SuffixArray::build("the quick brown fox");
        assert!(sa.contains("quick"));
        assert!(sa.contains("brown fox"));
    }

    #[test]
    fn contains_missing_returns_false() {
        let sa = SuffixArray::build("hello world");
        assert!(!sa.contains("zz"));
    }

    #[test]
    fn find_all_returns_positions() {
        let sa = SuffixArray::build("abcabcabc");
        let mut positions = sa.find_all("abc");
        positions.sort();
        assert_eq!(positions, vec![0, 3, 6]);
    }

    #[test]
    fn suffix_index_roundtrip() {
        let mut idx = SuffixIndex::new();
        idx.index_clip(1, "hello world");
        idx.index_clip(2, "goodbye world");
        let mut hits = idx.search("world");
        hits.sort();
        assert_eq!(hits, vec![1, 2]);
    }

    #[test]
    fn remove_clip_drops_entry() {
        let mut idx = SuffixIndex::new();
        idx.index_clip(1, "hello");
        idx.remove_clip(1);
        assert!(idx.search("hello").is_empty());
    }

    #[test]
    fn normalisation_case_insensitive() {
        let mut idx = SuffixIndex::new();
        idx.index_clip(1, "Hello World");
        assert_eq!(idx.search("world"), vec![1]);
    }

    #[test]
    fn content_longer_than_500_truncated() {
        let mut idx = SuffixIndex::new();
        let long: String = "a".repeat(600) + "UNIQUE";
        idx.index_clip(1, &long);
        assert!(idx.search("unique").is_empty());
    }

    #[test]
    fn empty_pattern_returns_false() {
        let sa = SuffixArray::build("hello");
        assert!(!sa.contains(""));
    }
}
