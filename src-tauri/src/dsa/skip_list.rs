use rand::Rng;

const MAX_LEVEL: usize = 16;
const P: f64 = 0.5;
const NULL: usize = usize::MAX;

struct SkipNode {
    score: f64,
    clip_id: i64,
    forward: Vec<usize>,
}

impl SkipNode {
    fn new(score: f64, clip_id: i64, level: usize) -> Self {
        Self {
            score,
            clip_id,
            forward: vec![NULL; level],
        }
    }
}

pub struct SkipList {
    nodes: Vec<Option<SkipNode>>,
    free: Vec<usize>,
    level: usize,
    len: usize,
}

impl SkipList {
    pub fn new() -> Self {
        let head = SkipNode::new(f64::MAX, i64::MIN, MAX_LEVEL);
        Self {
            nodes: vec![Some(head)],
            free: Vec::new(),
            level: 0,
            len: 0,
        }
    }

    fn alloc(&mut self, node: SkipNode) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    fn node(&self, idx: usize) -> &SkipNode {
        self.nodes[idx].as_ref().expect("node slot must be occupied")
    }

    fn node_mut(&mut self, idx: usize) -> &mut SkipNode {
        self.nodes[idx].as_mut().expect("node slot must be occupied")
    }

    fn precedes(&self, a: usize, score: f64, clip_id: i64) -> bool {
        let n = self.node(a);
        n.score > score || (n.score == score && n.clip_id > clip_id)
    }

    pub fn insert(&mut self, score: f64, clip_id: i64) {
        self.remove(clip_id);

        let new_level = random_level();
        if new_level > self.level {
            self.level = new_level;
        }

        let mut update = [0usize; MAX_LEVEL];
        let mut cur = 0usize;

        for i in (0..self.level).rev() {
            loop {
                let fwd = self.node(cur).forward[i];
                if fwd != NULL && self.precedes(fwd, score, clip_id) {
                    cur = fwd;
                } else {
                    break;
                }
            }
            update[i] = cur;
        }

        let new_idx = {
            let mut n = SkipNode::new(score, clip_id, new_level);
            for (i, &upd) in update.iter().enumerate().take(new_level) {
                n.forward[i] = self.node(upd).forward.get(i).copied().unwrap_or(NULL);
            }
            self.alloc(n)
        };

        for (i, &upd) in update.iter().enumerate().take(new_level) {
            self.node_mut(upd).forward[i] = new_idx;
        }

        self.len += 1;
    }

    pub fn remove(&mut self, clip_id: i64) -> bool {
        let target_score = {
            let mut found_score = None;
            let mut walk = self.node(0).forward[0];
            while walk != NULL {
                if self.node(walk).clip_id == clip_id {
                    found_score = Some(self.node(walk).score);
                    break;
                }
                walk = self.node(walk).forward[0];
            }
            match found_score {
                Some(s) => s,
                None => return false,
            }
        };

        let mut update = [0usize; MAX_LEVEL];
        let mut cur = 0usize;

        for i in (0..self.level).rev() {
            loop {
                let fwd = self.node(cur).forward[i];
                if fwd != NULL {
                    let n = self.node(fwd);
                    if n.score > target_score || (n.score == target_score && n.clip_id > clip_id) {
                        cur = fwd;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            update[i] = cur;
        }

        let target_idx = self.node(update[0]).forward[0];
        if target_idx == NULL || self.node(target_idx).clip_id != clip_id {
            return false;
        }

        for (i, &upd) in update.iter().enumerate().take(self.level) {
            let upd_fwd = self.node(upd).forward.get(i).copied().unwrap_or(NULL);
            if upd_fwd != target_idx {
                break;
            }
            let next = self.node(target_idx).forward.get(i).copied().unwrap_or(NULL);
            self.node_mut(upd).forward[i] = next;
        }

        self.nodes[target_idx] = None;
        self.free.push(target_idx);
        self.len -= 1;
        true
    }

    pub fn top_n(&self, n: usize) -> Vec<i64> {
        let mut result = Vec::new();
        let mut cur = self.node(0).forward[0];
        while cur != NULL && result.len() < n {
            result.push(self.node(cur).clip_id);
            cur = self.node(cur).forward[0];
        }
        result
    }

    pub fn iter_ranked(&self) -> Vec<i64> {
        let mut result = Vec::new();
        let mut cur = self.node(0).forward[0];
        while cur != NULL {
            result.push(self.node(cur).clip_id);
            cur = self.node(cur).forward[0];
        }
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn update_score(&mut self, clip_id: i64, new_score: f64) {
        self.remove(clip_id);
        self.insert(new_score, clip_id);
    }
}

impl Default for SkipList {
    fn default() -> Self {
        Self::new()
    }
}

fn random_level() -> usize {
    let mut rng = rand::thread_rng();
    let mut level = 1;
    while level < MAX_LEVEL && rng.gen::<f64>() < P {
        level += 1;
    }
    level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_single_top_n_returns_it() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 42);
        assert_eq!(sl.top_n(1), vec![42]);
    }

    #[test]
    fn insert_multiple_top_n_descending_order() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 1);
        sl.insert(3.0, 2);
        sl.insert(2.0, 3);
        let top = sl.top_n(3);
        assert_eq!(top[0], 2);
        assert_eq!(top[1], 3);
        assert_eq!(top[2], 1);
    }

    #[test]
    fn remove_existing_returns_true() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 10);
        assert!(sl.remove(10));
        assert!(!sl.top_n(10).contains(&10));
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 10);
        assert!(!sl.remove(99));
    }

    #[test]
    fn update_score_moves_entry() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 1);
        sl.insert(2.0, 2);
        sl.update_score(1, 3.0);
        let top = sl.top_n(2);
        assert_eq!(top[0], 1);
    }

    #[test]
    fn pinned_clip_appears_above_unpinned_same_recency() {
        let mut sl = SkipList::new();
        let base = 0.5f64;
        sl.insert(base + 0.5, 1);
        sl.insert(base, 2);
        let top = sl.top_n(2);
        assert_eq!(top[0], 1);
    }

    #[test]
    fn random_level_in_bounds() {
        for _ in 0..200 {
            let l = random_level();
            assert!((1..=MAX_LEVEL).contains(&l));
        }
    }

    #[test]
    fn iter_ranked_all_descending() {
        let mut sl = SkipList::new();
        sl.insert(3.0, 1);
        sl.insert(1.0, 2);
        sl.insert(2.0, 3);
        let all = sl.iter_ranked();
        assert_eq!(all, vec![1, 3, 2]);
    }

    #[test]
    fn len_and_is_empty() {
        let mut sl = SkipList::new();
        assert!(sl.is_empty());
        sl.insert(1.0, 1);
        assert_eq!(sl.len(), 1);
        sl.remove(1);
        assert!(sl.is_empty());
    }

    #[test]
    fn insert_duplicate_clip_id_updates_score() {
        let mut sl = SkipList::new();
        sl.insert(1.0, 5);
        sl.insert(4.0, 5);
        assert_eq!(sl.len(), 1);
        assert_eq!(sl.top_n(1), vec![5]);
    }
}
