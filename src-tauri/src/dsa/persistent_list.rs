use std::sync::Arc;

/// Immutable singly-linked list where each mutation returns a new version.
///
/// Nodes are shared between versions via `Arc`, so unchanged suffixes are
/// not copied. Cloning the list handle is O(1) and does not deep-copy.
pub struct PersistentList<T: Clone> {
    head: Option<Arc<Node<T>>>,
    pub len: usize,
}

struct Node<T> {
    value: T,
    next: Option<Arc<Node<T>>>,
}

impl<T: Clone> Clone for PersistentList<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
            len: self.len,
        }
    }
}

impl<T: Clone> Default for PersistentList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> PersistentList<T> {
    /// Create an empty persistent list.
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    /// Return a new version with `value` prepended to the front. O(1).
    pub fn prepend(&self, value: T) -> Self {
        let node = Arc::new(Node {
            value,
            next: self.head.clone(),
        });
        Self {
            head: Some(node),
            len: self.len + 1,
        }
    }

    /// Return a new version with the element at `index` removed. O(n).
    /// Returns a clone of the current version if the index is out of bounds.
    pub fn remove_at(&self, index: usize) -> Self {
        if index >= self.len {
            return self.clone();
        }

        let mut prefix: Vec<T> = Vec::with_capacity(index);
        let mut cur = self.head.clone();
        for _ in 0..index {
            match cur {
                Some(node) => {
                    prefix.push(node.value.clone());
                    cur = node.next.clone();
                }
                None => return self.clone(),
            }
        }

        let tail = match cur {
            Some(node) => node.next.clone(),
            None => return self.clone(),
        };

        let mut new_head = tail;
        for value in prefix.into_iter().rev() {
            new_head = Some(Arc::new(Node {
                value,
                next: new_head,
            }));
        }

        Self {
            head: new_head,
            len: self.len - 1,
        }
    }

    /// Return a reference to the element at `index`, or `None` if out of bounds. O(n).
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let mut cur = self.head.as_deref();
        for _ in 0..index {
            cur = cur.and_then(|n| n.next.as_deref());
        }
        cur.map(|n| &n.value)
    }

    /// Iterate over the elements of the current version in order. O(n).
    pub fn iter(&self) -> PersistentListIter<'_, T> {
        PersistentListIter {
            cur: self.head.as_deref(),
        }
    }

    /// Return the number of elements in the current version.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Borrowing iterator over a `PersistentList` version.
pub struct PersistentListIter<'a, T> {
    cur: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for PersistentListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.cur?;
        self.cur = node.next.as_deref();
        Some(&node.value)
    }
}

const MAX_VERSIONS: usize = 50;

/// Tracks a chain of `PersistentList` versions for undo support.
pub struct VersionHistory<T: Clone> {
    versions: Vec<PersistentList<T>>,
    current: usize,
}

impl<T: Clone> VersionHistory<T> {
    /// Create a history seeded with an initial version.
    pub fn new(initial: PersistentList<T>) -> Self {
        Self {
            versions: vec![initial],
            current: 0,
        }
    }

    /// Return the current version.
    pub fn current(&self) -> &PersistentList<T> {
        &self.versions[self.current]
    }

    /// Append a new version and advance the cursor to it.
    /// Discards any forward history past the current cursor.
    /// Caps retained versions at 50 — the oldest is dropped when over.
    pub fn push(&mut self, version: PersistentList<T>) {
        self.versions.truncate(self.current + 1);
        self.versions.push(version);
        if self.versions.len() > MAX_VERSIONS {
            self.versions.remove(0);
        }
        self.current = self.versions.len() - 1;
    }

    /// Move the cursor back one version and return the now-current version.
    /// Returns `None` if already at the oldest version.
    pub fn undo(&mut self) -> Option<&PersistentList<T>> {
        if self.current == 0 {
            return None;
        }
        self.current -= 1;
        Some(&self.versions[self.current])
    }

    /// Return true if `undo` would move to a previous version.
    pub fn can_undo(&self) -> bool {
        self.current > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepend_adds_to_front_without_mutating_original() {
        let a: PersistentList<i32> = PersistentList::new();
        let b = a.prepend(1);
        let c = b.prepend(2);
        assert_eq!(a.len(), 0);
        assert_eq!(b.len(), 1);
        assert_eq!(c.len(), 2);
        assert_eq!(b.get(0), Some(&1));
        assert_eq!(c.get(0), Some(&2));
        assert_eq!(c.get(1), Some(&1));
    }

    #[test]
    fn remove_at_produces_new_version_without_element() {
        let v = PersistentList::new().prepend(3).prepend(2).prepend(1);
        let r = v.remove_at(1);
        let original: Vec<i32> = v.iter().copied().collect();
        let removed: Vec<i32> = r.iter().copied().collect();
        assert_eq!(original, vec![1, 2, 3]);
        assert_eq!(removed, vec![1, 3]);
    }

    #[test]
    fn remove_at_out_of_bounds_returns_clone() {
        let v = PersistentList::new().prepend(1);
        let r = v.remove_at(5);
        assert_eq!(r.len(), v.len());
    }

    #[test]
    fn version_history_undo_returns_previous_version() {
        let v0 = PersistentList::new();
        let mut h = VersionHistory::new(v0.clone());
        let v1 = v0.prepend(1);
        h.push(v1.clone());
        let v2 = v1.prepend(2);
        h.push(v2);
        assert_eq!(h.current().len(), 2);
        let undone = h.undo().expect("undo should return previous version");
        assert_eq!(undone.len(), 1);
    }

    #[test]
    fn version_history_undo_at_oldest_returns_none() {
        let mut h: VersionHistory<i32> = VersionHistory::new(PersistentList::new());
        assert!(h.undo().is_none());
        assert!(!h.can_undo());
    }

    #[test]
    fn version_history_caps_at_fifty_versions() {
        let mut h: VersionHistory<i32> = VersionHistory::new(PersistentList::new());
        for i in 0..60 {
            let next = h.current().prepend(i);
            h.push(next);
        }
        assert_eq!(h.current().len(), 60);
        let mut undo_count = 0;
        while h.undo().is_some() {
            undo_count += 1;
        }
        assert_eq!(undo_count, MAX_VERSIONS - 1);
    }
}
