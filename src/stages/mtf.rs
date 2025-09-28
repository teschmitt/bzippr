use std::collections::{BTreeSet, VecDeque};

const RUNA: u8 = 100;
const RUNB: u8 = 101;

#[derive(Debug, PartialEq, Eq)]
pub struct MtfTransform {
    symbols: Vec<u8>,
    stack: Vec<u8>,
}

impl MtfTransform {
    /// Perform an MTF Transform on the passed data. As part of the transform, a second RLE
    /// pass is also performed. The "BZIP2: Format Specification" handbook says about this:
    /// "In practice, most implementations will combine the MTF and RLE2 stages"
    /// https://github.com/dsnet/compress/blob/39efe44ab707ffd2c1ef32cc7dbebfe584718686/doc/bzip2-format.pdf
    /// So that's what we're doing here:
    pub fn encode(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::empty();
        }

        let unique_bytes: BTreeSet<u8> = data.iter().copied().collect();
        let stack: Vec<u8> = unique_bytes.into_iter().collect();

        // MTF Transform
        let mut working_stack = stack.clone();
        let mut mtf_symbols = Vec::with_capacity(data.len());
        let mut current_byte = None;

        for &byte in data {
            if current_byte == Some(byte) {
                mtf_symbols.push(0);
                continue;
            }

            // remember index of current symbol in stack, then move it to front
            let position = working_stack
                .iter()
                .position(|&x| x == byte)
                .expect("Byte must exist in the stack");
            if position > 0 {
                // TODO: Check if this is really more performant than using a VecDeque with remove() and push_front()
                let val = working_stack[position];
                working_stack[0..=position].rotate_right(1);
                working_stack[0] = val;
            }

            mtf_symbols.push(position as u8);
            current_byte = Some(byte);
        }

        // RLE2 encoding
        let mut symbols = Vec::with_capacity(mtf_symbols.len());
        for chunk in mtf_symbols.chunk_by(|&a, &b| (a == 0 && b == 0) || (a != 0 && b != 0)) {
            if chunk[0] == 0 {
                emit_run(chunk.len(), &mut symbols);
            } else {
                symbols.extend(chunk);
            }
        }

        Self { symbols, stack }
    }

    pub fn empty() -> Self {
        Self {
            symbols: vec![],
            stack: vec![],
        }
    }
}

#[inline(always)]
fn emit_run(mut run_length: usize, out: &mut Vec<u8>) {
    while run_length > 0 {
        if run_length & 1 == 1 {
            out.push(RUNA);
            run_length = (run_length - 1) >> 1;
        } else {
            out.push(RUNB);
            run_length = (run_length - 2) >> 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(b"" => MtfTransform::empty(); "empty")]
    #[test_case(&vec![0] => MtfTransform { symbols: vec![100], stack: vec![0] }; "zero")]
    #[test_case(&vec![0, 0, 0, 0, 0, 0] => MtfTransform { symbols: vec![101, 101], stack: vec![0] }; "zeroes")]
    #[test_case(b"a" => MtfTransform {symbols: vec![100], stack: vec![97]}; "single byte")]
    #[test_case(b"abcdefg" => MtfTransform { symbols: vec![100, 1, 2, 3, 4, 5, 6], stack: vec![97, 98, 99, 100, 101, 102, 103] }; "all unique bytes")]
    #[test_case(b"aaaaabbbbbccccc" => MtfTransform { symbols: vec![100, 101, 1, 101, 100, 2, 101, 100], stack: vec![97, 98, 99] }; "repeated blocks")]
    #[test_case(b"aaaaa" => MtfTransform {symbols: vec![100, 101], stack: vec![ 97 ]}; "repeat same byte")]
    #[test_case(b"ababab" => MtfTransform {symbols: vec![100, 1, 1, 1, 1, 1], stack: vec![ 97, 98 ]}; "alternate two bytes")]
    #[test_case(b"abccbaabccba" => MtfTransform {symbols: vec![100, 1, 2, 100, 1, 2, 100, 1, 2, 100, 1, 2], stack: vec![ 97, 98, 99 ]}; "back and forth")]
    #[test_case(b"abacaba" => MtfTransform { symbols: vec![100, 1, 1, 2, 1, 2, 1], stack: vec![97, 98, 99] }; "overlapping patterns")]
    #[test_case(b"bbyaeeeeeeafeeeybzzzzzzzzzyz" => MtfTransform { symbols: vec![1, 100, 4, 2, 3, 100, 101, 1, 4, 2, 101, 3, 4, 5, 101, 100, 100, 2, 1], stack: vec![97, 98, 101, 102, 121, 122] }; "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    fn test_mtf_encode(data: &[u8]) -> MtfTransform {
        MtfTransform::encode(data)
    }
}
