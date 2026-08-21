use std::collections::HashMap;

use crate::storage::Clip;

/// Disjoint-set (Union-Find) with path compression and union by rank.
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
    /// Number of disjoint sets currently tracked.
    pub count: usize,
}

#[allow(dead_code)]
impl UnionFind {
    /// Create a structure with `n` singleton sets.
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
            count: n,
        }
    }

    /// Append a new singleton element and return its index.
    pub fn add_element(&mut self) -> usize {
        let idx = self.parent.len();
        self.parent.push(idx);
        self.rank.push(0);
        self.size.push(1);
        self.count += 1;
        idx
    }

    /// Find the root of `i`, applying path compression. Amortized O(alpha(n)).
    pub fn find(&mut self, i: usize) -> usize {
        let mut root = i;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cur = i;
        while self.parent[cur] != root {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    /// Merge the sets containing `i` and `j` by rank. Returns `false` if
    /// they were already in the same set.
    pub fn union(&mut self, i: usize, j: usize) -> bool {
        let ri = self.find(i);
        let rj = self.find(j);
        if ri == rj {
            return false;
        }

        let (small, large) = if self.rank[ri] < self.rank[rj] {
            (ri, rj)
        } else if self.rank[ri] > self.rank[rj] {
            (rj, ri)
        } else {
            self.rank[ri] += 1;
            (rj, ri)
        };

        self.parent[small] = large;
        self.size[large] += self.size[small];
        self.count -= 1;
        true
    }

    /// Return true if `i` and `j` share a root, without mutating state.
    pub fn connected(&self, i: usize, j: usize) -> bool {
        self.root_no_compress(i) == self.root_no_compress(j)
    }

    /// Return the size of the set containing `i`.
    pub fn set_size(&self, i: usize) -> usize {
        self.size[self.root_no_compress(i)]
    }

    fn root_no_compress(&self, mut i: usize) -> usize {
        while self.parent[i] != i {
            i = self.parent[i];
        }
        i
    }
}

/// Maps SQLite clip ids to Union-Find indices and exposes group queries.
pub struct ClipGroupManager {
    uf: UnionFind,
    clip_ids: Vec<i64>,
    id_to_index: HashMap<i64, usize>,
}

impl Default for ClipGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipGroupManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            uf: UnionFind::new(0),
            clip_ids: Vec::new(),
            id_to_index: HashMap::new(),
        }
    }

    /// Register a clip id as a new singleton set. No-op if already present.
    pub fn add_clip(&mut self, clip_id: i64) {
        if self.id_to_index.contains_key(&clip_id) {
            return;
        }
        let idx = self.uf.add_element();
        self.clip_ids.push(clip_id);
        self.id_to_index.insert(clip_id, idx);
    }

    /// Union the sets containing the two clip ids. No-op if either is unknown.
    pub fn group_clips(&mut self, clip_id_a: i64, clip_id_b: i64) {
        let ia = match self.id_to_index.get(&clip_id_a) {
            Some(&i) => i,
            None => return,
        };
        let ib = match self.id_to_index.get(&clip_id_b) {
            Some(&i) => i,
            None => return,
        };
        self.uf.union(ia, ib);
    }

    /// Return true if the two clip ids are in the same group.
    pub fn same_group(&mut self, clip_id_a: i64, clip_id_b: i64) -> bool {
        let ia = match self.id_to_index.get(&clip_id_a) {
            Some(&i) => i,
            None => return false,
        };
        let ib = match self.id_to_index.get(&clip_id_b) {
            Some(&i) => i,
            None => return false,
        };
        self.uf.find(ia) == self.uf.find(ib)
    }

    /// Return all clip ids in the same group as `clip_id`, including itself.
    pub fn get_group(&mut self, clip_id: i64) -> Vec<i64> {
        let idx = match self.id_to_index.get(&clip_id) {
            Some(&i) => i,
            None => return Vec::new(),
        };
        let root = self.uf.find(idx);
        let mut out = Vec::new();
        for i in 0..self.clip_ids.len() {
            if self.uf.find(i) == root {
                out.push(self.clip_ids[i]);
            }
        }
        out
    }

    /// Return all groups of size greater than one, each as a list of clip ids.
    pub fn all_groups(&mut self) -> Vec<Vec<i64>> {
        let n = self.clip_ids.len();
        let mut buckets: HashMap<usize, Vec<i64>> = HashMap::new();
        for i in 0..n {
            let root = self.uf.find(i);
            buckets.entry(root).or_default().push(self.clip_ids[i]);
        }
        buckets.into_values().filter(|g| g.len() > 1).collect()
    }

    /// Remove a clip id from the manager. Does not re-parent its children;
    /// the orphan index is left dangling but will not appear in group queries.
    pub fn remove_clip(&mut self, clip_id: i64) {
        if let Some(_idx) = self.id_to_index.remove(&clip_id) {
            if let Some(pos) = self.clip_ids.iter().position(|&id| id == clip_id) {
                self.clip_ids.remove(pos);
            }
        }
    }

    /// Return the count of disjoint sets across all tracked clips.
    pub fn group_count(&self) -> usize {
        self.uf.count
    }
}

/// Return true if `a` and `b` should be auto-grouped.
///
/// A pair is grouped when they share a non-empty `source_app` or when their
/// contents share a 30-character or longer prefix.
pub fn should_group(a: &Clip, b: &Clip) -> bool {
    if let (Some(app_a), Some(app_b)) = (&a.source_app, &b.source_app) {
        if !app_a.is_empty() && app_a == app_b {
            return true;
        }
    }
    shared_prefix_len(&a.content, &b.content) >= 30
}

fn shared_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_clip(id: i64, content: &str, source_app: Option<&str>) -> Clip {
        Clip {
            id,
            content: content.to_string(),
            content_type: "text".to_string(),
            created_at: "2026-04-23T00:00:00Z".to_string(),
            source_app: source_app.map(str::to_string),
            pinned: false,
            deleted_at: None,
            was_compressed: false,
            original_size: content.len() as i64,
            compressed_size: content.len() as i64,
            category: None,
            sender_name: None,
            sender_peer_id: None,
        }
    }

    #[test]
    fn find_returns_self_for_singleton() {
        let mut uf = UnionFind::new(5);
        assert_eq!(uf.find(0), 0);
        assert_eq!(uf.find(4), 4);
    }

    #[test]
    fn union_connects_two_elements() {
        let mut uf = UnionFind::new(3);
        assert!(uf.union(0, 1));
        assert_eq!(uf.find(0), uf.find(1));
        assert!(!uf.union(0, 1));
    }

    #[test]
    fn union_by_rank_higher_rank_wins() {
        let mut uf = UnionFind::new(4);
        uf.union(0, 1);
        uf.union(2, 3);
        uf.union(0, 2);
        let root = uf.find(0);
        assert_eq!(uf.find(1), root);
        assert_eq!(uf.find(2), root);
        assert_eq!(uf.find(3), root);
        assert_eq!(uf.count, 1);
    }

    #[test]
    fn path_compression_flattens_chain() {
        let mut uf = UnionFind::new(5);
        uf.union(0, 1);
        uf.union(2, 3);
        uf.union(0, 2);
        uf.union(0, 4);
        let root = uf.find(4);
        uf.find(3);
        for i in 0..5 {
            assert_eq!(uf.parent[i], root);
        }
    }

    #[test]
    fn connected_reflects_union_state() {
        let mut uf = UnionFind::new(3);
        assert!(!uf.connected(0, 1));
        uf.union(0, 1);
        assert!(uf.connected(0, 1));
        assert!(!uf.connected(0, 2));
    }

    #[test]
    fn clip_group_manager_roundtrip() {
        let mut m = ClipGroupManager::new();
        m.add_clip(10);
        m.add_clip(20);
        m.add_clip(30);
        m.group_clips(10, 20);
        let g = m.get_group(20);
        assert!(g.contains(&10) && g.contains(&20));
        assert!(!g.contains(&30));
        assert!(m.same_group(10, 20));
        assert!(!m.same_group(10, 30));
    }

    #[test]
    fn should_group_same_source_app_returns_true() {
        let a = mk_clip(1, "abc", Some("firefox"));
        let b = mk_clip(2, "zzz", Some("firefox"));
        assert!(should_group(&a, &b));
    }

    #[test]
    fn should_group_different_source_app_returns_false() {
        let a = mk_clip(1, "short", Some("firefox"));
        let b = mk_clip(2, "other", Some("terminal"));
        assert!(!should_group(&a, &b));
    }

    #[test]
    fn should_group_shared_prefix_returns_true() {
        let prefix = "https://example.com/path/to/resource";
        let a = mk_clip(1, &format!("{prefix}/first"), None);
        let b = mk_clip(2, &format!("{prefix}/second"), None);
        assert!(should_group(&a, &b));
    }

    #[test]
    fn should_group_short_shared_prefix_returns_false() {
        let a = mk_clip(1, "hello world", None);
        let b = mk_clip(2, "hello there", None);
        assert!(!should_group(&a, &b));
    }
}
