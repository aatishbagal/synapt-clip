use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use serde::{Deserialize, Serialize};

pub const COMPRESSION_THRESHOLD_BYTES: usize = 512;

#[derive(Debug, Clone)]
pub enum HuffmanNode {
    Leaf { char: char, freq: usize },
    Internal {
        freq: usize,
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

impl HuffmanNode {
    pub fn freq(&self) -> usize {
        match self {
            HuffmanNode::Leaf { freq, .. } => *freq,
            HuffmanNode::Internal { freq, .. } => *freq,
        }
    }
}

#[derive(Debug)]
struct HeapEntry {
    freq: usize,
    seq: usize,
    node: HuffmanNode,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq && self.seq == other.seq
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq
            .cmp(&other.freq)
            .then(self.seq.cmp(&other.seq))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HuffmanError {
    #[error("empty input")]
    EmptyInput,
    #[error("malformed encoded data: {0}")]
    Malformed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuffmanEncoded {
    pub bits: Vec<u8>,
    pub bit_length: usize,
    pub freq_table: Vec<(char, usize)>,
    pub original_len: usize,
}

pub fn build_tree(text: &str) -> Option<HuffmanNode> {
    if text.is_empty() {
        return None;
    }

    let mut freqs: HashMap<char, usize> = HashMap::new();
    for c in text.chars() {
        *freqs.entry(c).or_insert(0) += 1;
    }

    let mut leaves: Vec<(char, usize)> = freqs.into_iter().collect();
    leaves.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut heap: BinaryHeap<Reverse<HeapEntry>> = BinaryHeap::new();
    let mut seq = 0usize;
    for (c, f) in leaves {
        heap.push(Reverse(HeapEntry {
            freq: f,
            seq,
            node: HuffmanNode::Leaf { char: c, freq: f },
        }));
        seq += 1;
    }

    if heap.len() == 1 {
        let Reverse(only) = heap.pop().unwrap();
        let total = only.freq;
        return Some(HuffmanNode::Internal {
            freq: total,
            left: Box::new(only.node),
            right: Box::new(HuffmanNode::Leaf { char: '\0', freq: 0 }),
        });
    }

    while heap.len() > 1 {
        let Reverse(a) = heap.pop().unwrap();
        let Reverse(b) = heap.pop().unwrap();
        let merged_freq = a.freq + b.freq;
        let merged = HuffmanNode::Internal {
            freq: merged_freq,
            left: Box::new(a.node),
            right: Box::new(b.node),
        };
        heap.push(Reverse(HeapEntry {
            freq: merged_freq,
            seq,
            node: merged,
        }));
        seq += 1;
    }

    heap.pop().map(|Reverse(e)| e.node)
}

pub fn build_codes(root: &HuffmanNode) -> HashMap<char, Vec<bool>> {
    let mut codes = HashMap::new();
    let mut path = Vec::new();
    walk(root, &mut path, &mut codes);
    codes
}

fn walk(node: &HuffmanNode, path: &mut Vec<bool>, out: &mut HashMap<char, Vec<bool>>) {
    match node {
        HuffmanNode::Leaf { char, .. } => {
            out.insert(*char, path.clone());
        }
        HuffmanNode::Internal { left, right, .. } => {
            path.push(false);
            walk(left, path, out);
            path.pop();
            path.push(true);
            walk(right, path, out);
            path.pop();
        }
    }
}

pub fn encode(text: &str) -> Option<HuffmanEncoded> {
    if text.is_empty() {
        return None;
    }
    let tree = build_tree(text)?;
    let codes = build_codes(&tree);

    let mut freqs: HashMap<char, usize> = HashMap::new();
    for c in text.chars() {
        *freqs.entry(c).or_insert(0) += 1;
    }
    let mut freq_table: Vec<(char, usize)> = freqs.into_iter().collect();
    freq_table.sort_by(|a, b| a.0.cmp(&b.0));

    let mut bit_len = 0usize;
    let mut bits: Vec<u8> = Vec::new();
    for c in text.chars() {
        let code = codes.get(&c)?;
        for bit in code {
            let byte_idx = bit_len / 8;
            if byte_idx >= bits.len() {
                bits.push(0);
            }
            if *bit {
                bits[byte_idx] |= 1 << (bit_len % 8);
            }
            bit_len += 1;
        }
    }

    Some(HuffmanEncoded {
        bits,
        bit_length: bit_len,
        freq_table,
        original_len: text.chars().count(),
    })
}

pub fn decode(encoded: &HuffmanEncoded) -> Result<String, HuffmanError> {
    if encoded.freq_table.is_empty() {
        return Err(HuffmanError::Malformed("empty freq table".into()));
    }

    let mut leaves = encoded.freq_table.clone();
    leaves.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut heap: BinaryHeap<Reverse<HeapEntry>> = BinaryHeap::new();
    let mut seq = 0usize;
    for (c, f) in leaves {
        heap.push(Reverse(HeapEntry {
            freq: f,
            seq,
            node: HuffmanNode::Leaf { char: c, freq: f },
        }));
        seq += 1;
    }

    let root = if heap.len() == 1 {
        let Reverse(only) = heap
            .pop()
            .ok_or_else(|| HuffmanError::Malformed("empty heap".into()))?;
        let total = only.freq;
        HuffmanNode::Internal {
            freq: total,
            left: Box::new(only.node),
            right: Box::new(HuffmanNode::Leaf { char: '\0', freq: 0 }),
        }
    } else {
        while heap.len() > 1 {
            let Reverse(a) = heap
                .pop()
                .ok_or_else(|| HuffmanError::Malformed("heap underflow".into()))?;
            let Reverse(b) = heap
                .pop()
                .ok_or_else(|| HuffmanError::Malformed("heap underflow".into()))?;
            let merged_freq = a.freq + b.freq;
            let merged = HuffmanNode::Internal {
                freq: merged_freq,
                left: Box::new(a.node),
                right: Box::new(b.node),
            };
            heap.push(Reverse(HeapEntry {
                freq: merged_freq,
                seq,
                node: merged,
            }));
            seq += 1;
        }
        heap.pop()
            .map(|Reverse(e)| e.node)
            .ok_or_else(|| HuffmanError::Malformed("no root".into()))?
    };

    let mut out = String::with_capacity(encoded.original_len);
    let mut node = &root;
    for bit_pos in 0..encoded.bit_length {
        let byte_idx = bit_pos / 8;
        let bit_idx = bit_pos % 8;
        let byte = *encoded
            .bits
            .get(byte_idx)
            .ok_or_else(|| HuffmanError::Malformed("bits shorter than bit_length".into()))?;
        let bit = (byte >> bit_idx) & 1 == 1;

        node = match node {
            HuffmanNode::Internal { left, right, .. } => {
                if bit {
                    right
                } else {
                    left
                }
            }
            HuffmanNode::Leaf { .. } => {
                return Err(HuffmanError::Malformed("unexpected leaf during walk".into()));
            }
        };

        if let HuffmanNode::Leaf { char, .. } = node {
            out.push(*char);
            node = &root;
            if out.chars().count() == encoded.original_len {
                break;
            }
        }
    }

    if out.chars().count() != encoded.original_len {
        return Err(HuffmanError::Malformed(
            "decoded length does not match original".into(),
        ));
    }

    Ok(out)
}

pub fn maybe_encode(text: &str) -> (Vec<u8>, bool) {
    if text.len() <= COMPRESSION_THRESHOLD_BYTES {
        return (text.as_bytes().to_vec(), false);
    }
    match encode(text) {
        Some(encoded) => match serde_json::to_vec(&encoded) {
            Ok(bytes) => (bytes, true),
            Err(_) => (text.as_bytes().to_vec(), false),
        },
        None => (text.as_bytes().to_vec(), false),
    }
}

pub fn maybe_decode(data: &[u8], was_compressed: bool) -> Result<String, HuffmanError> {
    if !was_compressed {
        return std::str::from_utf8(data)
            .map(|s| s.to_string())
            .map_err(|e| HuffmanError::Malformed(format!("invalid utf-8: {e}")));
    }
    let encoded: HuffmanEncoded = serde_json::from_slice(data)
        .map_err(|e| HuffmanError::Malformed(format!("json parse failed: {e}")))?;
    decode(&encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_ascii() {
        let text = "the quick brown fox jumps over the lazy dog";
        let encoded = encode(text).expect("encodes");
        let decoded = decode(&encoded).expect("decodes");
        assert_eq!(decoded, text);
    }

    #[test]
    fn roundtrip_unicode() {
        let text = "café 日本語 emoji-free text";
        let encoded = encode(text).expect("encodes");
        let decoded = decode(&encoded).expect("decodes");
        assert_eq!(decoded, text);
    }

    #[test]
    fn more_frequent_chars_get_shorter_codes() {
        let text = "aaaaaaaaaaaaaaaaaaaabcd";
        let tree = build_tree(text).expect("tree");
        let codes = build_codes(&tree);
        let a_len = codes.get(&'a').unwrap().len();
        let d_len = codes.get(&'d').unwrap().len();
        assert!(a_len <= d_len);
    }

    #[test]
    fn empty_input_returns_none() {
        assert!(encode("").is_none());
        assert!(build_tree("").is_none());
    }

    #[test]
    fn decode_rejects_malformed() {
        let bad = HuffmanEncoded {
            bits: vec![],
            bit_length: 0,
            freq_table: vec![],
            original_len: 0,
        };
        assert!(decode(&bad).is_err());
    }

    #[test]
    fn maybe_encode_under_threshold_is_raw() {
        let short = "hello";
        let (bytes, was) = maybe_encode(short);
        assert!(!was);
        assert_eq!(bytes, short.as_bytes());
    }

    #[test]
    fn maybe_encode_over_threshold_compresses() {
        let long: String = "abcd ".repeat(200);
        let (_, was) = maybe_encode(&long);
        assert!(was);
    }

    #[test]
    fn maybe_decode_roundtrip_both_paths() {
        let short = "tiny string";
        let (raw_bytes, raw_flag) = maybe_encode(short);
        assert_eq!(maybe_decode(&raw_bytes, raw_flag).unwrap(), short);

        let long: String = "abcdefgh ".repeat(120);
        let (comp_bytes, comp_flag) = maybe_encode(&long);
        assert!(comp_flag);
        assert_eq!(maybe_decode(&comp_bytes, comp_flag).unwrap(), long);
    }

    #[test]
    fn repetitive_content_shrinks() {
        let text: String = "ababababab".repeat(100);
        let encoded = encode(&text).expect("encodes");
        assert!(encoded.bits.len() < text.len());
    }
}
