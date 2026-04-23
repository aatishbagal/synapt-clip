//! Compressed Trie (Patricia Trie) for prefix search on clip content.
//!
//! Each edge carries a string label rather than a single character, so
//! chains of single-child nodes are collapsed into one edge. This keeps
//! memory low for the long strings that frequently land on the clipboard.

use std::collections::HashMap;

/// Maximum content length indexed per clip. Longer clips are truncated
/// at the char boundary so a single large clip cannot dominate the Trie.
const MAX_INDEX_LEN: usize = 200;

struct TrieNode {
    children: HashMap<char, Box<TrieNode>>,
    edge_label: String,
    clip_ids: Vec<i64>,
    is_terminal: bool,
}

impl TrieNode {
    fn new(edge_label: String) -> Self {
        Self {
            children: HashMap::new(),
            edge_label,
            clip_ids: Vec::new(),
            is_terminal: false,
        }
    }
}

/// A compressed Trie mapping normalized content strings to clip ids.
pub struct Trie {
    root: TrieNode,
    size: usize,
}

impl Trie {
    /// Create an empty Trie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(String::new()),
            size: 0,
        }
    }

    /// Insert a clip's content. Content is lowercased and trimmed; content
    /// longer than 200 chars is truncated at the char boundary.
    pub fn insert(&mut self, content: &str, clip_id: i64) {
        let key = normalize(content);
        if key.is_empty() {
            return;
        }

        let key_chars: Vec<char> = key.chars().collect();
        Self::insert_at(&mut self.root, &key_chars, 0, clip_id);
        self.size += 1;
    }

    fn insert_at(node: &mut TrieNode, key: &[char], depth: usize, clip_id: i64) {
        if depth >= key.len() {
            node.is_terminal = true;
            if !node.clip_ids.contains(&clip_id) {
                node.clip_ids.push(clip_id);
            }
            return;
        }

        let first = key[depth];
        if let Some(child) = node.children.get_mut(&first) {
            let label_chars: Vec<char> = child.edge_label.chars().collect();
            let mut i = 0;
            while i < label_chars.len() && depth + i < key.len() && label_chars[i] == key[depth + i]
            {
                i += 1;
            }

            if i == label_chars.len() {
                Self::insert_at(child, key, depth + i, clip_id);
            } else {
                let common: String = label_chars[..i].iter().collect();
                let existing_suffix: String = label_chars[i..].iter().collect();

                let mut new_child = TrieNode::new(common);
                let mut old_child = std::mem::replace(child, Box::new(TrieNode::new(String::new())));
                old_child.edge_label = existing_suffix.clone();

                let existing_first = existing_suffix.chars().next().unwrap();
                new_child.children.insert(existing_first, old_child);

                if depth + i == key.len() {
                    new_child.is_terminal = true;
                    new_child.clip_ids.push(clip_id);
                } else {
                    let rem: String = key[depth + i..].iter().collect();
                    let rem_first = rem.chars().next().unwrap();
                    let mut leaf = TrieNode::new(rem);
                    leaf.is_terminal = true;
                    leaf.clip_ids.push(clip_id);
                    new_child.children.insert(rem_first, Box::new(leaf));
                }

                *child = Box::new(new_child);
            }
        } else {
            let rem: String = key[depth..].iter().collect();
            let mut leaf = TrieNode::new(rem);
            leaf.is_terminal = true;
            leaf.clip_ids.push(clip_id);
            node.children.insert(first, Box::new(leaf));
        }
    }

    /// Return all clip ids whose indexed content starts with `prefix`.
    pub fn prefix_search(&self, prefix: &str) -> Vec<i64> {
        let key = normalize(prefix);
        if key.is_empty() {
            return Vec::new();
        }
        let key_chars: Vec<char> = key.chars().collect();

        let mut node = &self.root;
        let mut consumed = 0;

        while consumed < key_chars.len() {
            let first = key_chars[consumed];
            let Some(child) = node.children.get(&first) else {
                return Vec::new();
            };
            let label_chars: Vec<char> = child.edge_label.chars().collect();
            let remaining = &key_chars[consumed..];

            let to_compare = label_chars.len().min(remaining.len());
            for i in 0..to_compare {
                if label_chars[i] != remaining[i] {
                    return Vec::new();
                }
            }

            if remaining.len() < label_chars.len() {
                // Prefix ends inside this edge — every id in subtree matches.
                return collect_ids(child);
            }

            consumed += label_chars.len();
            node = child;
        }

        collect_ids(node)
    }

    /// Remove a clip id from the Trie. Nodes holding other ids are kept.
    pub fn remove(&mut self, clip_id: i64) {
        remove_id(&mut self.root, clip_id);
    }

    /// Number of distinct insertions performed.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Remove everything.
    pub fn clear(&mut self) {
        self.root = TrieNode::new(String::new());
        self.size = 0;
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize(s: &str) -> String {
    let trimmed = s.trim().to_lowercase();
    if trimmed.chars().count() <= MAX_INDEX_LEN {
        trimmed
    } else {
        trimmed.chars().take(MAX_INDEX_LEN).collect()
    }
}

fn collect_ids(node: &TrieNode) -> Vec<i64> {
    let mut out = Vec::new();
    let mut stack: Vec<&TrieNode> = vec![node];
    while let Some(n) = stack.pop() {
        for id in &n.clip_ids {
            if !out.contains(id) {
                out.push(*id);
            }
        }
        for child in n.children.values() {
            stack.push(child);
        }
    }
    out
}

fn remove_id(node: &mut TrieNode, clip_id: i64) {
    node.clip_ids.retain(|id| *id != clip_id);
    if node.clip_ids.is_empty() {
        node.is_terminal = false;
    }

    let child_keys: Vec<char> = node.children.keys().copied().collect();
    for k in child_keys {
        if let Some(child) = node.children.get_mut(&k) {
            remove_id(child, clip_id);
            if !child.is_terminal && child.children.is_empty() {
                node.children.remove(&k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_prefix_search_roundtrip() {
        let mut t = Trie::new();
        t.insert("hello world", 1);
        assert_eq!(t.prefix_search("hello"), vec![1]);
    }

    #[test]
    fn multiple_ids_share_prefix() {
        let mut t = Trie::new();
        t.insert("foobar", 1);
        t.insert("foobaz", 2);
        let mut ids = t.prefix_search("foo");
        ids.sort();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn unknown_prefix_returns_empty() {
        let mut t = Trie::new();
        t.insert("hello", 1);
        assert!(t.prefix_search("xyz").is_empty());
    }

    #[test]
    fn remove_drops_id() {
        let mut t = Trie::new();
        t.insert("hello", 1);
        t.insert("hello", 2);
        t.remove(1);
        assert_eq!(t.prefix_search("hello"), vec![2]);
    }

    #[test]
    fn shared_edge_is_compressed() {
        let mut t = Trie::new();
        t.insert("abc", 1);
        t.insert("abcdef", 2);

        // Walking from root by first char 'a' should land on an edge starting with 'a'.
        let first = t.root.children.get(&'a').expect("root has 'a' edge");
        // The edge label covers the shared 'abc' prefix (length 3).
        assert_eq!(first.edge_label, "abc");
        // And it has a child for the 'def' suffix of the longer string.
        assert!(first.children.contains_key(&'d'));
    }

    #[test]
    fn normalisation_case_insensitive() {
        let mut t = Trie::new();
        t.insert("Hello", 1);
        assert_eq!(t.prefix_search("hello"), vec![1]);
        assert_eq!(t.prefix_search("HELLO"), vec![1]);
    }

    #[test]
    fn empty_insert_does_not_panic() {
        let mut t = Trie::new();
        t.insert("", 1);
        t.insert("   ", 2);
        assert_eq!(t.size(), 0);
    }

    #[test]
    fn long_content_is_truncated() {
        let mut t = Trie::new();
        let long: String = "x".repeat(300);
        t.insert(&long, 1);
        let short: String = "x".repeat(200);
        assert_eq!(t.prefix_search(&short), vec![1]);
    }

    #[test]
    fn clear_empties_trie() {
        let mut t = Trie::new();
        t.insert("hello", 1);
        t.clear();
        assert_eq!(t.size(), 0);
        assert!(t.prefix_search("hello").is_empty());
    }
}
