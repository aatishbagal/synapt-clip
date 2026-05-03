use std::collections::HashSet;

use crate::search::bloom::BloomFilter;
use crate::search::fuzzy::FuzzySearcher;
use crate::search::suffix_array::SuffixIndex;
use crate::search::trie::Trie;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Prefix,
    Substring,
    Fuzzy,
}

impl MatchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchType::Prefix => "prefix",
            MatchType::Substring => "substring",
            MatchType::Fuzzy => "fuzzy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub clip_id: i64,
    pub score: f32,
    pub match_type: MatchType,
    pub match_positions: Vec<(usize, usize)>,
}

pub struct SearchEngine {
    trie: Trie,
    suffix_index: SuffixIndex,
    fuzzy: FuzzySearcher,
    bloom: BloomFilter,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            trie: Trie::new(),
            suffix_index: SuffixIndex::new(),
            fuzzy: FuzzySearcher::new(2),
            bloom: BloomFilter::new(1000, 0.01),
        }
    }

    pub fn build_from_clips(&mut self, clips: &[(i64, String)]) {
        for (id, content) in clips {
            self.index_clip(*id, content);
        }
    }

    pub fn index_clip(&mut self, clip_id: i64, content: &str) {
        self.trie.insert(content, clip_id);
        self.suffix_index.index_clip(clip_id, content);
        self.bloom.insert(content);
    }

    pub fn remove_clip(&mut self, clip_id: i64, _content: &str) {
        self.trie.remove(clip_id);
        self.suffix_index.remove_clip(clip_id);
    }

    pub fn is_probable_duplicate(&self, content: &str) -> bool {
        self.bloom.probably_contains(content)
    }

    pub fn search(
        &self,
        query: &str,
        all_clips: &[(i64, String, bool)],
        recent_ids: &[i64],
    ) -> Vec<SearchResult> {
        let q = query.trim();
        if q.is_empty() {
            return Vec::new();
        }
        let q_lower = q.to_lowercase();
        let q_char_count = q_lower.chars().count();

        let pinned: HashSet<i64> = all_clips
            .iter()
            .filter_map(|(id, _, p)| if *p { Some(*id) } else { None })
            .collect();
        let recent: HashSet<i64> = recent_ids.iter().take(20).copied().collect();

        let content_by_id = |target: i64| -> Option<&String> {
            all_clips
                .iter()
                .find(|(id, _, _)| *id == target)
                .map(|(_, c, _)| c)
        };

        let mut results: Vec<SearchResult> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();

        for id in self.trie.prefix_search(q) {
            if seen.insert(id) {
                let end = content_by_id(id)
                    .map(|c| {
                        let c_lower = c.to_lowercase();
                        let take = c_lower
                            .char_indices()
                            .nth(q_char_count)
                            .map(|(i, _)| i)
                            .unwrap_or(c_lower.len());
                        take
                    })
                    .unwrap_or(q.len());
                let mut score = 5.0f32;
                if pinned.contains(&id) {
                    score += 10.0;
                }
                if recent.contains(&id) {
                    score += 1.0;
                }
                results.push(SearchResult {
                    clip_id: id,
                    score,
                    match_type: MatchType::Prefix,
                    match_positions: vec![(0, end)],
                });
            }
        }

        for id in self.suffix_index.search(q) {
            if seen.insert(id) {
                let positions = content_by_id(id)
                    .and_then(|c| first_match_range(c, &q_lower))
                    .map(|r| vec![r])
                    .unwrap_or_default();
                let mut score = 3.0f32;
                if pinned.contains(&id) {
                    score += 10.0;
                }
                if recent.contains(&id) {
                    score += 1.0;
                }
                results.push(SearchResult {
                    clip_id: id,
                    score,
                    match_type: MatchType::Substring,
                    match_positions: positions,
                });
            }
        }

        if results.is_empty() {
            let fuzzy_clips: Vec<(i64, String)> = all_clips
                .iter()
                .map(|(id, c, _)| (*id, c.clone()))
                .collect();
            for (id, distance) in self.fuzzy.search(q, &fuzzy_clips) {
                if seen.insert(id) {
                    let mut score = 2.0f32 - (distance as f32) * 0.5;
                    if pinned.contains(&id) {
                        score += 10.0;
                    }
                    if recent.contains(&id) {
                        score += 1.0;
                    }
                    results.push(SearchResult {
                        clip_id: id,
                        score,
                        match_type: MatchType::Fuzzy,
                        match_positions: Vec::new(),
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn first_match_range(content: &str, pattern_lower: &str) -> Option<(usize, usize)> {
    if pattern_lower.is_empty() {
        return None;
    }
    let lower = content.to_lowercase();
    let byte_start = lower.find(pattern_lower)?;
    let byte_end = byte_start + pattern_lower.len();
    let char_start = content[..byte_start.min(content.len())].chars().count();
    let char_end = content[..byte_end.min(content.len())].chars().count();
    Some((char_start, char_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(id: i64, content: &str, pinned: bool) -> (i64, String, bool) {
        (id, content.to_string(), pinned)
    }

    #[test]
    fn prefix_hit_takes_priority() {
        let mut eng = SearchEngine::new();
        eng.index_clip(1, "hello world");
        eng.index_clip(2, "say hello to the world");
        let clips = vec![clip(1, "hello world", false), clip(2, "say hello to the world", false)];
        let results = eng.search("hello", &clips, &[]);
        assert!(!results.is_empty());
        assert_eq!(results[0].clip_id, 1);
        assert_eq!(results[0].match_type, MatchType::Prefix);
    }

    #[test]
    fn substring_when_no_prefix() {
        let mut eng = SearchEngine::new();
        eng.index_clip(1, "the quick brown fox");
        let clips = vec![clip(1, "the quick brown fox", false)];
        let results = eng.search("quick", &clips, &[]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Substring);
    }

    #[test]
    fn fuzzy_fallback_when_empty() {
        let mut eng = SearchEngine::new();
        eng.index_clip(1, "please receive the letter");
        let clips = vec![clip(1, "please receive the letter", false)];
        let results = eng.search("recieve", &clips, &[]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Fuzzy);
    }

    #[test]
    fn pinned_boost_raises_score() {
        let mut eng = SearchEngine::new();
        eng.index_clip(1, "hello a");
        eng.index_clip(2, "hello b");
        let clips = vec![clip(1, "hello a", false), clip(2, "hello b", true)];
        let results = eng.search("hello", &clips, &[]);
        let pinned_score = results.iter().find(|r| r.clip_id == 2).map(|r| r.score).unwrap_or(0.0);
        let other_score = results.iter().find(|r| r.clip_id == 1).map(|r| r.score).unwrap_or(0.0);
        assert!(pinned_score > other_score);
    }

    #[test]
    fn is_probable_duplicate_true_after_insert() {
        let mut eng = SearchEngine::new();
        eng.index_clip(1, "hello");
        assert!(eng.is_probable_duplicate("hello"));
    }

    #[test]
    fn empty_query_returns_nothing() {
        let eng = SearchEngine::new();
        let results = eng.search("", &[], &[]);
        assert!(results.is_empty());
    }
}
