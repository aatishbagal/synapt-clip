use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

struct Entry<T> {
    item: T,
    seq: u64,
}

impl<T: Ord> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.seq == other.seq
    }
}
impl<T: Ord> Eq for Entry<T> {}
impl<T: Ord> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Ord> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.item.cmp(&other.item).then(self.seq.cmp(&other.seq))
    }
}

pub struct Depq<T: Ord + Clone> {
    max_heap: BinaryHeap<Entry<T>>,
    min_heap: BinaryHeap<Reverse<Entry<T>>>,
    dead: HashSet<u64>,
    next_seq: u64,
    len: usize,
}

impl<T: Ord + Clone> Depq<T> {
    pub fn new() -> Self {
        Self {
            max_heap: BinaryHeap::new(),
            min_heap: BinaryHeap::new(),
            dead: HashSet::new(),
            next_seq: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, item: T) {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.max_heap.push(Entry {
            item: item.clone(),
            seq,
        });
        self.min_heap.push(Reverse(Entry { item, seq }));
        self.len += 1;
    }

    pub fn peek_min(&mut self) -> Option<&T> {
        self.flush_min();
        self.min_heap.peek().map(|Reverse(e)| &e.item)
    }

    pub fn peek_max(&mut self) -> Option<&T> {
        self.flush_max();
        self.max_heap.peek().map(|e| &e.item)
    }

    pub fn pop_min(&mut self) -> Option<T> {
        self.flush_min();
        let Reverse(entry) = self.min_heap.pop()?;
        self.dead.insert(entry.seq);
        self.len -= 1;
        Some(entry.item)
    }

    pub fn pop_max(&mut self) -> Option<T> {
        self.flush_max();
        let entry = self.max_heap.pop()?;
        self.dead.insert(entry.seq);
        self.len -= 1;
        Some(entry.item)
    }

    fn flush_min(&mut self) {
        while let Some(Reverse(e)) = self.min_heap.peek() {
            if self.dead.remove(&e.seq) {
                self.min_heap.pop();
            } else {
                break;
            }
        }
    }

    fn flush_max(&mut self) {
        while let Some(e) = self.max_heap.peek() {
            if self.dead.remove(&e.seq) {
                self.max_heap.pop();
            } else {
                break;
            }
        }
    }
}

impl<T: Ord + Clone> Default for Depq<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_returns_both_ends() {
        let mut d: Depq<i32> = Depq::new();
        d.push(3);
        d.push(1);
        d.push(5);
        d.push(2);
        assert_eq!(d.peek_min(), Some(&1));
        assert_eq!(d.peek_max(), Some(&5));
    }

    #[test]
    fn pop_min_returns_smallest() {
        let mut d: Depq<i32> = Depq::new();
        for v in [3, 1, 5, 2, 4] {
            d.push(v);
        }
        assert_eq!(d.pop_min(), Some(1));
        assert_eq!(d.pop_min(), Some(2));
    }

    #[test]
    fn pop_max_returns_largest() {
        let mut d: Depq<i32> = Depq::new();
        for v in [3, 1, 5, 2, 4] {
            d.push(v);
        }
        assert_eq!(d.pop_max(), Some(5));
        assert_eq!(d.pop_max(), Some(4));
    }

    #[test]
    fn interleaved_pushes_and_pops() {
        let mut d: Depq<i32> = Depq::new();
        d.push(10);
        d.push(1);
        assert_eq!(d.pop_min(), Some(1));
        d.push(5);
        d.push(20);
        assert_eq!(d.pop_max(), Some(20));
        assert_eq!(d.pop_min(), Some(5));
        assert_eq!(d.pop_max(), Some(10));
        assert!(d.is_empty());
    }

    #[test]
    fn single_element_both_ends_equal() {
        let mut d: Depq<i32> = Depq::new();
        d.push(42);
        assert_eq!(d.peek_min(), Some(&42));
        assert_eq!(d.peek_max(), Some(&42));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn empty_depq_returns_none() {
        let mut d: Depq<i32> = Depq::new();
        assert_eq!(d.pop_min(), None);
        assert_eq!(d.pop_max(), None);
        assert!(d.is_empty());
    }
}
