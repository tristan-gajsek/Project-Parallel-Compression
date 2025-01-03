use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use bit_vec::BitVec;

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

    pub fn get_byte(&self, bits: &BitVec) -> u8 {
        unimplemented!()
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

pub fn compress(input: &[u8]) -> Box<[u8]> {
    let mut counts = HashMap::new();
    input
        .iter()
        .for_each(|byte| *counts.entry(byte).or_insert(0u32) += 1);
    let tree = Node::new_tree(
        counts
            .iter()
            .map(|(byte, count)| Node::new(*count, **byte))
            .collect(),
    );
    unimplemented!()
}

pub fn decompress(input: &[u8]) -> Box<[u8]> {
    unimplemented!()
}
