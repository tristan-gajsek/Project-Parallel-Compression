use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use bit_vec::BitVec;

#[derive(Debug, PartialEq, Eq)]
struct Node {
    value: u32,
    byte: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new(value: u32, byte: u8) -> Self {
        Self {
            value,
            byte: Some(byte),
            left: None,
            right: None,
        }
    }

    pub fn new_tree(mut counts: BinaryHeap<Node>) -> Self {
        while counts.len() > 1 {
            let right = counts.pop().unwrap();
            let left = counts.pop().unwrap();
            let node = Node {
                value: left.value + right.value,
                byte: None,
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
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

    fn get_codes_inner(&self, mut code: BitVec, codes: &mut HashMap<u8, BitVec>) {
        self.byte.map(|b| codes.insert(b, code.clone()));
        let mut code2 = code.clone();
        code.push(false);
        code2.push(true);
        self.left.as_ref().map(|l| l.get_codes_inner(code, codes));
        self.right.as_ref().map(|r| r.get_codes_inner(code2, codes));
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
        other
            .value
            .cmp(&self.value)
            .then(other.byte.cmp(&self.byte))
    }
}

pub fn compress(input: &[u8]) -> Box<[u8]> {
    let mut counts = HashMap::new();
    input
        .iter()
        .for_each(|byte| *counts.entry(byte).or_insert(0u32) += 1);
    let tree = Node::new_tree(
        counts
            .into_iter()
            .map(|(byte, count)| Node::new(count, *byte))
            .collect(),
    );
    dbg!(&tree);
    dbg!(tree.get_codes());
    unimplemented!()
}

pub fn decompress(input: &[u8]) -> Box<[u8]> {
    unimplemented!()
}
