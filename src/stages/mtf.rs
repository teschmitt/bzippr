use std::collections::BTreeSet;

const RUNA: usize = 1337;
const RUNB: usize = 1338;

#[derive(Debug, PartialEq, Eq)]
pub struct MtfTransform {
    // TODO: these aren't symbols, they're indices ... maybe make that clear in the naming
    symbols: Vec<usize>,
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

            mtf_symbols.push(position);
            current_byte = Some(byte);
        }

        // RLE2 encoding
        let mut symbols: Vec<usize> = Vec::with_capacity(mtf_symbols.len());
        for chunk in mtf_symbols.chunk_by(|&a, &b| (a == 0 && b == 0) || (a != 0 && b != 0)) {
            if chunk[0] == 0 {
                emit_run(chunk.len(), &mut symbols);
            } else {
                symbols.extend(chunk);
            }
        }

        Self { symbols, stack }
    }

    pub fn decode(&self) -> Vec<u8> {
        if self.is_empty() {
            return Vec::new();
        }

        // RLE2 decoding pass
        let mut mtf_indices = Vec::with_capacity(self.symbols.len() * 2);
        let mut run_length = 0;
        let mut power = 1;

        for &idx in &self.symbols {
            match idx {
                RUNA => {
                    run_length += power;
                    // TODO: test potential overflow here
                    power <<= 1;
                }
                RUNB => {
                    run_length += power * 2;
                    power <<= 1;
                }
                found_index => {
                    if run_length > 0 {
                        mtf_indices.extend(std::iter::repeat(0).take(run_length));
                        run_length = 0;
                        power = 1;
                    }
                    mtf_indices.push(found_index);
                }
            }
        }

        if run_length > 0 {
            mtf_indices.extend(std::iter::repeat(0).take(run_length));
        }

        // MTF decoding pass
        let mut result = Vec::with_capacity(mtf_indices.len());
        let mut working_stack = self.stack.clone();

        for &idx in &mtf_indices {
            let symbol = working_stack.get(idx).expect("Invalid index in MTF decode");
            result.push(*symbol);

            if idx > 0 {
                let val = working_stack[idx];
                working_stack[0..=idx].rotate_right(1);
                working_stack[0] = val;
            }
        }

        result
    }

    pub fn empty() -> Self {
        Self {
            symbols: vec![],
            stack: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.len() == 0
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }
}

#[inline(always)]
fn emit_run(mut run_length: usize, out: &mut Vec<usize>) {
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
    #[test_case(&vec![0] => MtfTransform { symbols: vec![RUNA], stack: vec![0] }; "zero")]
    #[test_case(&vec![0, 0, 0, 0, 0, 0] => MtfTransform { symbols: vec![RUNB, RUNB], stack: vec![0] }; "zeroes")]
    #[test_case(b"a" => MtfTransform {symbols: vec![RUNA], stack: vec![97]}; "single byte")]
    #[test_case(b"abcdefg" => MtfTransform { symbols: vec![RUNA, 1, 2, 3, 4, 5, 6], stack: vec![97, 98, 99, 100, 101, 102, 103] }; "all unique bytes")]
    #[test_case(b"gab" => MtfTransform { symbols: vec![2, 1, 2], stack: vec![97, 98, 103] }; "no runs")]
    #[test_case(b"aaaaabbbbbccccc" => MtfTransform { symbols: vec![RUNA, RUNB, 1, RUNB, RUNA, 2, RUNB, RUNA], stack: vec![97, 98, 99] }; "repeated blocks")]
    #[test_case(b"aaaaa" => MtfTransform {symbols: vec![RUNA, RUNB], stack: vec![ 97 ]}; "repeat same byte")]
    #[test_case(b"ababab" => MtfTransform {symbols: vec![RUNA, 1, 1, 1, 1, 1], stack: vec![ 97, 98 ]}; "alternate two bytes")]
    #[test_case(b"abccbaabccba" => MtfTransform {symbols: vec![RUNA, 1, 2, RUNA, 1, 2, RUNA, 1, 2, RUNA, 1, 2], stack: vec![ 97, 98, 99 ]}; "back and forth")]
    #[test_case(b"abacaba" => MtfTransform { symbols: vec![RUNA, 1, 1, 2, 1, 2, 1], stack: vec![97, 98, 99] }; "overlapping patterns")]
    #[test_case(b"bbyaeeeeeeafeeeybzzzzzzzzzyz" => MtfTransform { symbols: vec![1, RUNA, 4, 2, 3, RUNA, RUNB, 1, 4, 2, RUNB, 3, 4, 5, RUNB, RUNA, RUNA, 2, 1], stack: vec![97, 98, 101, 102, 121, 122] }; "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    #[test_case(b"abccc" => MtfTransform { symbols: vec![RUNA, 1, 2, RUNB], stack: vec![97, 98, 99] }; "one runb at end")]
    #[test_case(b"abcccc" => MtfTransform { symbols: vec![RUNA, 1, 2, RUNA, RUNA], stack: vec![97, 98, 99] }; "runas at end")]
    fn test_mtf_encode(data: &[u8]) -> MtfTransform {
        MtfTransform::encode(data)
    }

    #[test_case(MtfTransform::empty() => Vec::<u8>::new(); "empty")]
    #[test_case(MtfTransform { symbols: vec![RUNA], stack: vec![0] } => vec![0u8]; "zero")]
    #[test_case(MtfTransform { symbols: vec![RUNB, RUNB], stack: vec![0] } => vec![0, 0, 0, 0, 0, 0]; "zeroes")]
    #[test_case(MtfTransform {symbols: vec![RUNA], stack: vec![97]} => b"a".to_vec(); "single byte")]
    #[test_case(MtfTransform { symbols: vec![RUNA, 1, 2, 3, 4, 5, 6], stack: vec![97, 98, 99, 100, 101, 102, 103] } => b"abcdefg".to_vec(); "all unique bytes")]
    #[test_case(MtfTransform { symbols: vec![2, 1, 2], stack: vec![97, 98, 103] } => b"gab".to_vec(); "no runs")]
    #[test_case(MtfTransform { symbols: vec![RUNA, RUNB, 1, RUNB, RUNA, 2, RUNB, RUNA], stack: vec![97, 98, 99] } => b"aaaaabbbbbccccc".to_vec(); "repeated blocks")]
    #[test_case(MtfTransform {symbols: vec![RUNA, RUNB], stack: vec![ 97 ]} => b"aaaaa".to_vec(); "repeat same byte")]
    #[test_case(MtfTransform {symbols: vec![RUNA, 1, 1, 1, 1, 1], stack: vec![ 97, 98 ]} => b"ababab".to_vec(); "alternate two bytes")]
    #[test_case(MtfTransform {symbols: vec![RUNA, 1, 2, RUNA, 1, 2, RUNA, 1, 2, RUNA, 1, 2], stack: vec![ 97, 98, 99 ]} => b"abccbaabccba".to_vec(); "back and forth")]
    #[test_case(MtfTransform { symbols: vec![RUNA, 1, 1, 2, 1, 2, 1], stack: vec![97, 98, 99] } => b"abacaba".to_vec(); "overlapping patterns")]
    #[test_case(MtfTransform { symbols: vec![1, RUNA, 4, 2, 3, RUNA, RUNB, 1, 4, 2, RUNB, 3, 4, 5, RUNB, RUNA, RUNA, 2, 1], stack: vec![97, 98, 101, 102, 121, 122] } => b"bbyaeeeeeeafeeeybzzzzzzzzzyz".to_vec(); "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    #[test_case(MtfTransform { symbols: vec![RUNA, 1, 2, RUNB], stack: vec![97, 98, 99] } => b"abccc".to_vec(); "one runb at end")]
    #[test_case(MtfTransform { symbols: vec![RUNA, 1, 2, RUNA, RUNA], stack: vec![97, 98, 99] } => b"abcccc".to_vec(); "runas at end")]
    fn test_mtf_decode(mtf: MtfTransform) -> Vec<u8> {
        mtf.decode()
    }

    // TODO: tests with corrupted data, e.g. indexes out of bounds
}
