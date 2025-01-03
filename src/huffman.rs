use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    io::Cursor,
};

use anyhow::{anyhow, bail, Ok, Result};
use bit_vec::BitVec;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug, PartialEq, Eq)]
struct Node {
    value: u32,
    content: NodeContent,
}

#[derive(Debug, PartialEq, Eq)]
enum NodeContent {
    Byte(u8),
    Leaves(Option<Box<Node>>, Option<Box<Node>>),
}

impl Node {
    pub fn new(value: u32, byte: u8) -> Self {
        Self {
            value,
            content: NodeContent::Byte(byte),
        }
    }

    pub fn new_tree(mut counts: BinaryHeap<Node>) -> Self {
        while counts.len() > 1 {
            let left = counts.pop().unwrap();
            let right = counts.pop().unwrap();
            let node = Node {
                value: left.value + right.value,
                content: NodeContent::Leaves(Some(Box::new(left)), Some(Box::new(right))),
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
                codes.insert(*byte, code.clone());
            }
            NodeContent::Leaves(left, right) => {
                let (mut lcode, mut rcode) = (code.clone(), code);
                lcode.push(false);
                rcode.push(true);
                left.as_ref().map(|l| l.get_codes_inner(lcode, codes));
                right.as_ref().map(|r| r.get_codes_inner(rcode, codes));
            }
        }
    }

    pub fn get_byte(&self, bits: &BitVec) -> Result<u8> {
        let mut node = self;
        for bit in bits {
            match &node.content {
                NodeContent::Byte(byte) => return Ok(*byte),
                NodeContent::Leaves(left, right) => {
                    node = match bit {
                        false => left,
                        true => right,
                    }
                    .as_ref()
                    .ok_or(anyhow!("Invalid bit pattern for Huffman tree: {bits}"))?;
                }
            }
        }
        bail!("Couldn't find byte for {bits} in Huffman tree");
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.value.cmp(&self.value)
    }
}

fn counts_to_header(counts: &HashMap<u8, u32>) -> Vec<u8> {
    let mut header = Vec::with_capacity(2 + 5 * counts.len());

    header.extend_from_slice(&(counts.len() as u16).to_be_bytes());
    for (byte, count) in counts.iter() {
        header.extend_from_slice(&[*byte]);
        header.extend_from_slice(&count.to_be_bytes());
    }
    header
}

fn header_to_counts(header: &[u8]) -> Result<HashMap<u8, u32>> {
    let mut counts = HashMap::new();

    let mut cursor = Cursor::new(header);
    for _ in 0..cursor.read_u16::<BigEndian>()? {
        counts.insert(cursor.read_u8()?, cursor.read_u32::<BigEndian>()?);
    }
    Ok(counts)
}

pub fn compress(input: &[u8]) -> Result<Box<[u8]>> {
    let mut counts = HashMap::new();
    input
        .iter()
        .for_each(|byte| *counts.entry(*byte).or_insert(0u32) += 1);
    let codes = Node::new_tree(
        counts
            .iter()
            .map(|(byte, count)| Node::new(*count, *byte))
            .collect(),
    )
    .get_codes();

    let mut header = counts_to_header(&counts);
    header.append(
        &mut input
            .iter()
            .flat_map(|byte| {
                codes
                    .get(byte)
                    .cloned()
                    .ok_or(anyhow!("Couldn't find byte {byte} in Huffman table"))
            })
            .flatten()
            .collect::<BitVec>()
            .to_bytes(),
    );
    Ok(header.into_boxed_slice())
}

pub fn decompress(input: &[u8]) -> Box<[u8]> {
    unimplemented!()
}
