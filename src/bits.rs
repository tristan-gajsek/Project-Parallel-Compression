pub struct BitWriter {
    bytes: Vec<u8>,
    bits: u8,
    current_bit: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter {
            bytes: Vec::new(),
            bits: 0,
            current_bit: 7,
        }
    }

    pub fn into_boxed_slice(self) -> Box<[u8]> {
        self.bytes.into_boxed_slice()
    }

    pub fn byte(&mut self, byte: u8) {
        self.bits(byte, 8);
    }

    pub fn bits(&mut self, bits: u8, count: u8) {
        for i in (0..count).rev() {
            self.bit(((bits >> i) & 1) == 1);
        }
    }

    pub fn bit(&mut self, bit: bool) {
        if bit {
            self.bits |= 1 << self.current_bit;
        }

        if self.current_bit != 0 {
            self.current_bit -= 1;
        } else {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.current_bit < 7 {
            self.bytes.push(self.bits);
            self.bits = 0;
            self.current_bit = 7;
        }
    }
}

pub struct BitReader<'a> {
    bytes: &'a [u8],
    index: usize,
    bit: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        BitReader {
            bytes: slice,
            index: 0,
            bit: 7,
        }
    }

    pub fn bit(&mut self) -> bool {
        let bit = self.bytes[self.index] & (1 << self.bit);
        if self.bit != 0 {
            self.bit -= 1;
        } else {
            self.bit = 7;
            self.index += 1;
        }
        bit != 0
    }

    pub fn bits(&mut self, count: u8) -> u8 {
        let mut bits = 0;
        for i in (0..count).rev() {
            bits |= (self.bit() as u8) << i;
        }
        bits
    }

    pub fn byte(&mut self) -> u8 {
        self.bits(8)
    }
}
