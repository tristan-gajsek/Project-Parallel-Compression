use std::sync::LazyLock;

use crate::bits::{BitReader, BitWriter};

static DIFFS: LazyLock<[Vec<i16>; 4]> = LazyLock::new(|| {
    [
        (-2..=-1).chain(1..=2).collect(),
        (-6..=-3).chain(3..=6).collect(),
        (-14..=-7).chain(7..=14).collect(),
        (-30..=-15).chain(15..=30).collect(),
    ]
});

#[rustfmt::skip]
fn diff_to_bits(diff: i16) -> (u8, u8) {
    match diff.abs() {
        1..=2 => ((0b00 << 2) | DIFFS[0].binary_search(&diff).unwrap() as u8, 4),
        3..=6 => ((0b01 << 3) | DIFFS[1].binary_search(&diff).unwrap() as u8, 5),
        7..=14 => ((0b10 << 4) | DIFFS[2].binary_search(&diff).unwrap() as u8, 6),
        15..=30 => ((0b11 << 5) | DIFFS[3].binary_search(&diff).unwrap() as u8, 7),
        _ => unreachable!(),
    }
}

fn bits_to_diff(bits: u8, count: u8) -> i16 {
    let index = bits as usize;
    match count - 2 {
        0b00 => DIFFS[0][index],
        0b01 => DIFFS[1][index],
        0b10 => DIFFS[2][index],
        0b11 => DIFFS[3][index],
        _ => unreachable!(),
    }
}

pub fn compress(slice: &[u8]) -> Vec<u8> {
    let mut bits = BitWriter::new();
    let mut repetition = 0;
    bits.byte(slice[0]);

    (1..slice.len()).for_each(|i| match slice[i] as i16 - slice[i - 1] as i16 {
        0 => {
            repetition += 1;
            if repetition == 8 {
                bits.bits(0b01, 2);
                bits.bits(repetition - 1, 3);
                repetition = 0;
            }
        }
        diff @ _ => {
            if repetition != 0 {
                bits.bits(0b01, 2);
                bits.bits(repetition - 1, 3);
                repetition = 0;
            }

            if diff.abs() <= 30 {
                bits.bits(0b00, 2);
                let (data, count) = diff_to_bits(diff);
                bits.bits(data, count);
            } else {
                bits.bits(0b10, 2);
                bits.bit(diff.is_negative());
                bits.byte(diff.unsigned_abs() as u8);
            }
        }
    });

    if repetition != 0 {
        bits.bits(0b01, 2);
        bits.bits(repetition - 1, 3);
    }
    bits.bits(0b11, 2);
    bits.flush();
    bits.into()
}

pub fn decompress(slice: &[u8]) -> Vec<u8> {
    let mut bits = BitReader::new(slice);
    let mut bytes = Vec::from([bits.byte()]);
    let mut last_byte = bytes[0];

    loop {
        match bits.bits(2) {
            0b00 => {
                let bit_count = bits.bits(2) + 2;
                let byte = last_byte as i16 + bits_to_diff(bits.bits(bit_count), bit_count);
                bytes.push(byte as u8);
                last_byte = byte as u8;
            }
            0b01 => {
                for _ in 0..=bits.bits(3) {
                    bytes.push(last_byte);
                }
            }
            0b10 => {
                let sign: i16 = if bits.bit() { -1 } else { 1 };
                let byte = (last_byte as i16 + sign * bits.byte() as i16) as u8;
                bytes.push(byte);
                last_byte = byte;
            }
            0b11 => break,
            _ => unreachable!(),
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_and_decompression() {
        let text = include_bytes!("main.rs");
        assert_eq!(text.as_ref(), decompress(&compress(text)));
    }

    #[test]
    fn separated_compression_and_decompression() {
        let chunks = include_bytes!("main.rs").chunks(32).collect::<Vec<_>>();
        let compressed = chunks.iter().map(|c| compress(*c)).collect::<Vec<_>>();
        let decompressed = compressed.iter().map(|c| decompress(c)).collect::<Vec<_>>();
        assert_eq!(chunks, decompressed);
    }
}
