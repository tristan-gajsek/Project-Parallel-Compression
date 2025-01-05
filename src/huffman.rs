use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap, HashMap},
    io::Cursor,
};

use anyhow::{Ok, Result};
use bitvec::{bitvec, order::Msb0};
use byteorder::{BigEndian, ReadBytesExt};

type BitVec = bitvec::vec::BitVec<u8, Msb0>;

#[derive(Debug, PartialEq, Eq)]
struct Node {
    value: u32,
    content: NodeContent,
}

#[derive(Debug, PartialEq, Eq)]
enum NodeContent {
    Byte(u8),
    Branches(Option<Box<Node>>, Option<Box<Node>>),
}

impl NodeContent {
    pub fn byte(&self) -> Option<u8> {
        match self {
            NodeContent::Byte(byte) => Some(*byte),
            NodeContent::Branches(..) => None,
        }
    }
}

impl Node {
    pub fn new(value: u32, byte: u8) -> Self {
        Self {
            value,
            content: NodeContent::Byte(byte),
        }
    }

    pub fn new_tree(counts: &BTreeMap<u8, u32>) -> Self {
        let mut counts: BinaryHeap<_> = counts
            .iter()
            .map(|(byte, count)| Node::new(*count, *byte))
            .collect();

        while counts.len() > 1 {
            let left = counts.pop().unwrap();
            let right = counts.pop().unwrap();
            let node = Node {
                value: left.value + right.value,
                content: NodeContent::Branches(Some(Box::new(left)), Some(Box::new(right))),
            };
            counts.push(node);
        }
        counts.pop().unwrap()
    }

    pub fn get_codes(&self) -> HashMap<u8, BitVec> {
        let mut codes = HashMap::new();
        self.get_codes_inner(BitVec::new(), &mut codes);
        codes
    }

    fn get_codes_inner(&self, code: BitVec, codes: &mut HashMap<u8, BitVec>) {
        match &self.content {
            NodeContent::Byte(byte) => {
                codes.insert(
                    *byte,
                    match code.is_empty() {
                        false => code,
                        true => bitvec!(u8, Msb0; 0),
                    },
                );
            }
            NodeContent::Branches(left, right) => {
                let (mut lcode, mut rcode) = (code.clone(), code);
                lcode.push(false);
                rcode.push(true);
                left.as_ref().map(|l| l.get_codes_inner(lcode, codes));
                right.as_ref().map(|r| r.get_codes_inner(rcode, codes));
            }
        }
    }

    pub fn get_byte(&self, bits: &BitVec) -> Option<u8> {
        let mut node = self;
        for bit in bits {
            match &node.content {
                NodeContent::Byte(byte) => return Some(*byte),
                NodeContent::Branches(left, right) => {
                    node = if !bit { left } else { right }.as_ref()?;
                }
            }
        }
        node.content.byte()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .value
            .cmp(&self.value)
            .then_with(|| other.content.cmp(&self.content))
    }
}

impl PartialOrd for NodeContent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeContent {
    fn cmp(&self, other: &Self) -> Ordering {
        use NodeContent as N;
        match (self, other) {
            (N::Branches(..), N::Branches(..)) => Ordering::Equal,
            (N::Byte(b1), N::Byte(b2)) => b1.cmp(b2),
            (N::Byte(_), N::Branches(..)) => Ordering::Less,
            (N::Branches(..), N::Byte(_)) => Ordering::Greater,
        }
    }
}

fn counts_to_header(counts: &BTreeMap<u8, u32>) -> Vec<u8> {
    let mut header = Vec::with_capacity(2 + 5 * counts.len());

    header.extend_from_slice(&(counts.len() as u16).to_be_bytes());
    for (byte, count) in counts.iter() {
        header.extend_from_slice(&[*byte]);
        header.extend_from_slice(&count.to_be_bytes());
    }
    header
}

fn header_to_counts(header: &[u8]) -> Result<(BTreeMap<u8, u32>, BitVec)> {
    let mut counts = BTreeMap::new();

    let mut cursor = Cursor::new(header);
    for _ in 0..cursor.read_u16::<BigEndian>()? {
        counts.insert(cursor.read_u8()?, cursor.read_u32::<BigEndian>()?);
    }
    let bit_count = cursor.read_u64::<BigEndian>()?;
    let mut bits = BitVec::from_slice(&header[cursor.position() as usize..]);
    bits.truncate(bit_count as usize);
    Ok((counts, bits))
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut counts = BTreeMap::new();
    input
        .iter()
        .for_each(|byte| *counts.entry(*byte).or_insert(0_u32) += 1);
    let codes = Node::new_tree(&counts).get_codes();

    let mut compressed = counts_to_header(&counts);
    let mut bits = BitVec::new();
    for byte in input {
        bits.append(&mut codes.get(byte).cloned().unwrap());
    }
    compressed.extend_from_slice(&(bits.len() as u64).to_be_bytes());
    compressed.append(&mut bits.into_vec());
    compressed
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    let (counts, bits) = header_to_counts(input)?;
    let tree = Node::new_tree(&counts);
    let mut decompressed = vec![];

    let mut pattern = BitVec::new();
    for bit in bits {
        pattern.push(bit);
        if let Some(byte) = tree.get_byte(&pattern) {
            decompressed.push(byte);
            pattern.clear();
        }
    }
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use bitvec::bitvec;

    use super::*;

    #[test]
    fn header_creation_and_parsing() {
        let counts = BTreeMap::from([(1, 1), (2, 2), (3, 3)]);
        let mut header = counts_to_header(&counts);
        header.extend_from_slice(&24_u64.to_be_bytes());
        header.extend_from_slice(&[1, 2, 3]);
        let (new_counts, bits) = header_to_counts(&header).expect("Header parsing failed");
        assert_eq!(counts, new_counts);
        assert_eq!(bits, BitVec::from_slice(&[1, 2, 3]))
    }

    #[test]
    fn get_bytes_from_tree() {
        let tree = Node::new_tree(&BTreeMap::from([(1, 1), (2, 2), (3, 3)]));
        dbg!(&tree);
        assert_eq!(tree.get_byte(&bitvec!(u8, Msb0; 1, 0)), Some(1));
        assert_eq!(tree.get_byte(&bitvec!(u8, Msb0; 1, 1)), Some(2));
        assert_eq!(tree.get_byte(&bitvec!(u8, Msb0; 0)), Some(3));
    }

    #[test]
    fn compression_and_decompression() {
        let text = include_bytes!("main.rs");
        let compressed = compress(text);
        let decompressed = decompress(&compressed).expect("Decompression failed");
        assert_eq!(text, decompressed.as_slice());
    }

    #[test]
    fn separated_compression_and_decompression() {
        let chunks = include_bytes!("main.rs").chunks(32).collect::<Vec<_>>();
        let compressed = chunks.iter().map(|c| compress(*c)).collect::<Vec<_>>();
        let decompressed = compressed
            .iter()
            .map(|c| decompress(c).expect("Decompression failed"))
            .collect::<Vec<_>>();
        assert_eq!(chunks, decompressed);
    }
}
