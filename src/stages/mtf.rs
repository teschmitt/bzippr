use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
pub enum MtfIndex {
    RunA,
    RunB,
    Val(u8),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MtfTransform {
    indices: Vec<MtfIndex>,
    stack: Vec<u8>,
}

impl MtfTransform {
    /// Perform an MTF Transform on the passed data. As part of the transform, a second RLE
    /// pass is also performed. The "BZIP2: Format Specification" handbook says about this:
    /// "In practice, most implementations will combine the MTF and RLE2 stages"
    /// https://github.com/dsnet/compress/blob/39efe44ab707ffd2c1ef32cc7dbebfe584718686/doc/bzip2-format.pdf
    /// So that's what we're doing here:
    pub fn encode(data: &[u8]) -> Self {
        // TODO: think long and hard if the input to the decode shouldn't rather be a BwtEncoded
        if data.is_empty() {
            return Self::empty();
        }

        let unique_bytes: BTreeSet<u8> = data.iter().copied().collect();
        let stack: Vec<u8> = unique_bytes.into_iter().collect();

        // MTF Transform
        let mut working_stack = stack.clone();
        let mut mtf_indices = Vec::with_capacity(data.len());
        let mut current_byte = None;

        for &byte in data {
            if current_byte == Some(byte) {
                mtf_indices.push(0);
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

            mtf_indices.push(position as u8);
            current_byte = Some(byte);
        }

        // RLE2 encoding
        let mut indices: Vec<MtfIndex> = Vec::with_capacity(mtf_indices.len());
        for chunk in mtf_indices.chunk_by(|&a, &b| (a == 0 && b == 0) || (a != 0 && b != 0)) {
            if chunk[0] == 0 {
                emit_run(chunk.len(), &mut indices);
            } else {
                indices.extend(chunk.iter().map(|&i| MtfIndex::Val(i)));
            }
        }

        Self { indices, stack }
    }

    pub fn decode(&self) -> Vec<u8> {
        if self.is_empty() {
            return Vec::new();
        }

        // RLE2 decoding pass
        let mut mtf_indices = Vec::with_capacity(self.indices.len() * 2); // TODO: Find a better way to estimate required capacity
        let mut run_length = 0;
        let mut power = 1;

        for idx in &self.indices {
            match idx {
                MtfIndex::RunA => {
                    run_length += power;
                    // TODO: test potential overflow of power here
                    power <<= 1;
                }
                MtfIndex::RunB => {
                    run_length += power * 2;
                    power <<= 1;
                }
                MtfIndex::Val(found_index) => {
                    if run_length > 0 {
                        mtf_indices.extend(std::iter::repeat(0).take(run_length));
                        run_length = 0;
                        power = 1;
                    }
                    mtf_indices.push(*found_index);
                }
            }
        }

        if run_length > 0 {
            mtf_indices.extend(std::iter::repeat(0).take(run_length));
        }

        // MTF decoding pass
        let mut result = Vec::with_capacity(mtf_indices.len());
        let mut working_stack = self.stack.clone();

        for idx in mtf_indices.iter().map(|&i| i as usize) {
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
            indices: vec![],
            stack: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.len() == 0
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn num_stack(&self) -> usize {
        self.stack.len()
    }

    pub fn indices(&self) -> &Vec<MtfIndex> {
        &self.indices
    }
}

#[inline(always)]
fn emit_run(mut run_length: usize, out: &mut Vec<MtfIndex>) {
    while run_length > 0 {
        if run_length & 1 == 1 {
            out.push(MtfIndex::RunA);
            run_length = (run_length - 1) >> 1;
        } else {
            out.push(MtfIndex::RunB);
            run_length = (run_length - 2) >> 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    mod t {
        pub const RUNA: usize = 1337;
        pub const RUNB: usize = 1338;
    }

    #[test_case(b"" => (vec![], vec![]); "empty")]
    #[test_case(&vec![0] => (vec![t::RUNA], vec![0]); "zero")]
    #[test_case(&vec![0, 0, 0, 0, 0, 0] => (vec![t::RUNB, t::RUNB], vec![0]); "zeroes")]
    #[test_case(b"a" => (vec![t::RUNA], vec![97]); "single byte")]
    #[test_case(b"abcdefg" => (vec![t::RUNA, 1, 2, 3, 4, 5, 6], vec![97, 98, 99, 100, 101, 102, 103]); "all unique bytes")]
    #[test_case(b"gab" => (vec![2, 1, 2], vec![97, 98, 103]); "no runs")]
    #[test_case(b"aaaaabbbbbccccc" => (vec![t::RUNA, t::RUNB, 1, t::RUNB, t::RUNA, 2, t::RUNB, t::RUNA], vec![97, 98, 99]); "repeated blocks")]
    #[test_case(b"aaaaa" => (vec![t::RUNA, t::RUNB], vec![ 97 ]); "repeat same byte")]
    #[test_case(b"ababab" => (vec![t::RUNA, 1, 1, 1, 1, 1], vec![ 97, 98 ]); "alternate two bytes")]
    #[test_case(b"abccbaabccba" => (vec![t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2], vec![ 97, 98, 99 ]); "back and forth")]
    #[test_case(b"abacaba" => (vec![t::RUNA, 1, 1, 2, 1, 2, 1], vec![97, 98, 99]); "overlapping patterns")]
    #[test_case(b"bbyaeeeeeeafeeeybzzzzzzzzzyz" => (vec![1, t::RUNA, 4, 2, 3, t::RUNA, t::RUNB, 1, 4, 2, t::RUNB, 3, 4, 5, t::RUNB, t::RUNA, t::RUNA, 2, 1], vec![97, 98, 101, 102, 121, 122]); "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    #[test_case(b"abccc" => (vec![t::RUNA, 1, 2, t::RUNB], vec![97, 98, 99]); "one runb at end")]
    #[test_case(b"abcccc" => (vec![t::RUNA, 1, 2, t::RUNA, t::RUNA], vec![97, 98, 99]); "runas at end")]
    fn test_mtf_encode(data: &[u8]) -> (Vec<usize>, Vec<u8>) {
        let mtf = MtfTransform::encode(data);
        (
            mtf.indices
                .iter()
                .map(|i| match i {
                    MtfIndex::RunA => t::RUNA,
                    MtfIndex::RunB => t::RUNB,
                    MtfIndex::Val(v) => *v as usize,
                })
                .collect(),
            mtf.stack,
        )
    }

    #[test_case(Vec::<usize>::new(), Vec::<u8>::new() => Vec::<u8>::new(); "empty")]
    #[test_case(vec![t::RUNA], vec![0] => vec![0]; "zero")]
    #[test_case(vec![t::RUNB, t::RUNB], vec![0] => vec![0, 0, 0, 0, 0, 0]; "zeroes")]
    #[test_case(vec![t::RUNA], vec![97] => b"a".to_vec(); "single byte")]
    #[test_case(vec![t::RUNA, 1, 2, 3, 4, 5, 6], vec![97, 98, 99, 100, 101, 102, 103] => b"abcdefg".to_vec(); "all unique bytes")]
    #[test_case(vec![2, 1, 2], vec![97, 98, 103] => b"gab".to_vec(); "no runs")]
    #[test_case(vec![t::RUNA, t::RUNB, 1, t::RUNB, t::RUNA, 2, t::RUNB, t::RUNA], vec![97, 98, 99] => b"aaaaabbbbbccccc".to_vec(); "repeated blocks")]
    #[test_case(vec![t::RUNA, t::RUNB], vec![ 97 ] => b"aaaaa".to_vec(); "repeat same byte")]
    #[test_case(vec![t::RUNA, 1, 1, 1, 1, 1], vec![ 97, 98 ] => b"ababab".to_vec(); "alternate two bytes")]
    #[test_case(vec![t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2], vec![ 97, 98, 99 ] => b"abccbaabccba".to_vec(); "back and forth")]
    #[test_case(vec![t::RUNA, 1, 1, 2, 1, 2, 1], vec![97, 98, 99] => b"abacaba".to_vec(); "overlapping patterns")]
    #[test_case(vec![1, t::RUNA, 4, 2, 3, t::RUNA, t::RUNB, 1, 4, 2, t::RUNB, 3, 4, 5, t::RUNB, t::RUNA, t::RUNA, 2, 1], vec![97, 98, 101, 102, 121, 122] => b"bbyaeeeeeeafeeeybzzzzzzzzzyz".to_vec(); "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    #[test_case(vec![t::RUNA, 1, 2, t::RUNB], vec![97, 98, 99] => b"abccc".to_vec(); "one runb at end")]
    #[test_case(vec![t::RUNA, 1, 2, t::RUNA, t::RUNA], vec![97, 98, 99] => b"abcccc".to_vec(); "runas at end")]
    fn test_mtf_decode(indices: Vec<usize>, stack: Vec<u8>) -> Vec<u8> {
        let mtf = MtfTransform {
            indices: indices
                .iter()
                .map(|&i| match i {
                    t::RUNA => MtfIndex::RunA,
                    t::RUNB => MtfIndex::RunB,
                    v => MtfIndex::Val(v as u8),
                })
                .collect(),
            stack,
        };
        mtf.decode()
    }

    // TODO: tests with corrupted data, e.g. indexes out of bounds
}
