use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
pub struct MtfTransform {
    symbols: Vec<u8>,
    stack: Vec<u8>,
}

impl MtfTransform {
    pub fn encode(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::empty();
        }

        let unique_bytes: BTreeSet<u8> = data.iter().copied().collect();
        let stack: Vec<u8> = unique_bytes.clone().into_iter().collect();

        let mut working_stack: VecDeque<u8> = stack.clone().into_iter().collect();
        let mut symbols = Vec::with_capacity(data.len());
        let mut current_byte = None;

        for &byte in data {
            if current_byte == Some(byte) {
                symbols.push(0);
                continue;
            }

            let position = working_stack
                .iter()
                .position(|&x| x == byte)
                .expect("Byte must exist in the stack");

            working_stack.remove(position);
            working_stack.push_front(byte);
            symbols.push(position as u8);

            current_byte = Some(byte);
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(b"bbyaeeeeeeafeeeybzzzzzzzzzyz" => MtfTransform { symbols: vec![1, 0, 4, 2, 3, 0, 0, 0, 0, 0, 1, 4, 2, 0, 0, 3, 4, 5, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1], stack: vec![97, 98, 101, 102, 121, 122] }; "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    fn test_mtf_encode(data: &[u8]) -> MtfTransform {
        MtfTransform::encode(data)
    }
}
