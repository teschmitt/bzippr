#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RleRun {
    pub(crate) value: u8,
    pub(crate) count: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RleSequence(Vec<RleRun>);

impl RleSequence {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn empty() -> Self {
        RleSequence(Vec::new())
    }
}

impl From<&[u8]> for RleSequence {
    fn from(data: &[u8]) -> Self {
        let mut sequence = Vec::new();
        if !data.is_empty() {
            let mut current_value = data[0];
            let mut current_count = 1;

            for &byte in &data[1..] {
                if byte == current_value {
                    current_count += 1;
                } else {
                    sequence.push(RleRun {
                        value: current_value,
                        count: current_count,
                    });
                    current_value = byte;
                    current_count = 1;
                }
            }

            sequence.push(RleRun {
                value: current_value,
                count: current_count,
            });
        }

        RleSequence(sequence)
    }
}

impl Into<Vec<u8>> for RleSequence {
    fn into(self) -> Vec<u8> {
        let mut data = Vec::new();
        for run in self.0 {
            data.extend(std::iter::repeat(run.value).take(run.count as usize));
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("" => RleSequence::empty(); "empty input")]
    #[test_case("a" => RleSequence(vec![RleRun { value: b'a', count: 1 }]); "single a")]
    #[test_case("aaaaaaaaaa" => RleSequence(vec![RleRun { value: b'a', count: 10 }]); "ten as")]
    #[test_case("ab" => RleSequence(vec![RleRun { value: b'a', count: 1 }, RleRun { value: b'b', count: 1 }]); "one and one")]
    #[test_case("ba" => RleSequence(vec![RleRun { value: b'b', count: 1 }, RleRun { value: b'a', count: 1 }]); "one and one reverse")]
    #[test_case("aaaaabbbbb" => RleSequence(vec![RleRun { value: b'a', count: 5 }, RleRun { value: b'b', count: 5 }]); "five and five")]
    #[test_case("bbbbbaaaaa" => RleSequence(vec![RleRun { value: b'b', count: 5 }, RleRun { value: b'a', count: 5 }]); "five and five reverse")]
    #[test_case("abc" => RleSequence(vec![RleRun { value: b'a', count: 1 }, RleRun { value: b'b', count: 1 }, RleRun { value: b'c', count: 1 }]); "abc")]
    #[test_case("cba" => RleSequence(vec![RleRun { value: b'c', count: 1 }, RleRun { value: b'b', count: 1 }, RleRun { value: b'a', count: 1 }]); "abc reverse")]
    fn test_correct_encoding(data: &str) -> RleSequence {
        RleSequence::from(data.as_bytes())
    }

    #[test_case(RleSequence::empty() => Vec::<u8>::new(); "empty input")]
    #[test_case(RleSequence(vec![RleRun { value: b'a', count: 1 }]) => vec![b'a']; "single a")]
    #[test_case(RleSequence(vec![RleRun { value: b'a', count: 10 }]) => vec![b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a']; "ten as")]
    #[test_case(RleSequence(vec![RleRun { value: b'a', count: 1 }, RleRun { value: b'b', count: 1 }]) => vec![b'a', b'b']; "one and one")]
    #[test_case(RleSequence(vec![RleRun { value: b'b', count: 1 }, RleRun { value: b'a', count: 1 }]) => vec![b'b', b'a']; "one and one reverse")]
    #[test_case(RleSequence(vec![RleRun { value: b'a', count: 5 }, RleRun { value: b'b', count: 5 }]) => vec![b'a', b'a', b'a', b'a', b'a', b'b', b'b', b'b', b'b', b'b']; "five and five")]
    #[test_case(RleSequence(vec![RleRun { value: b'b', count: 5 }, RleRun { value: b'a', count: 5 }]) => vec![b'b', b'b', b'b', b'b', b'b', b'a', b'a', b'a', b'a', b'a']; "five and five reverse")]
    #[test_case(RleSequence(vec![RleRun { value: b'a', count: 1 }, RleRun { value: b'b', count: 1 }, RleRun { value: b'c', count: 1 }]) => vec![b'a', b'b', b'c']; "abc")]
    #[test_case(RleSequence(vec![RleRun { value: b'c', count: 1 }, RleRun { value: b'b', count: 1 }, RleRun { value: b'a', count: 1 }]) => vec![b'c', b'b', b'a']; "abc reverse")]

    fn test_correct_decoding(seq: RleSequence) -> Vec<u8> {
        seq.into()
    }

    #[test_case("" => 0; "empty input")]
    #[test_case("a" => 1; "single a")]
    #[test_case("ab" => 2; "two singles")]
    #[test_case("aaaaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbbbbbbbbb" => 2; "two runs")]
    #[test_case("aaaaaaaaaaabbbbbbbbbbbbbaaaaaaaaaaaaaaaa" => 3; "three runs")]
    #[test_case("abcdefgaaaaaaaaaa" => 8; "eight runs")]
    #[test_case("ababababab" => 10; "alternating pattern")]
    #[test_case("abcabcabcabc" => 12; "repeating abc pattern")]
    fn test_correct_length(data: &str) -> usize {
        let rle_sequence = RleSequence::from(data.as_bytes());
        rle_sequence.len()
    }

    #[test_case(&[0, 0, 0, 255, 255, 1] => RleSequence(vec![RleRun { value: 0, count: 3 }, RleRun { value: 255, count: 2 }, RleRun { value: 1, count: 1 }]); "binary data")]
    fn test_binary_data(data: &[u8]) -> RleSequence {
        RleSequence::from(data)
    }

    #[test_case(""; "empty")]
    #[test_case("a"; "single")]
    #[test_case("aaaaabbbbbccccc"; "three runs")]
    #[test_case("abcdefghijklmnop"; "no compression")]
    fn test_round_trip_property(original: &str) {
        let original_bytes = original.as_bytes();
        let rle_sequence = RleSequence::from(original_bytes);
        let decoded: Vec<u8> = rle_sequence.into();
        assert_eq!(original_bytes, decoded.as_slice());
    }

    #[test]
    fn test_large_count_values() {
        let large_data = vec![b'x'; 100_000];
        let rle_sequence = RleSequence::from(large_data.as_slice());
        assert_eq!(rle_sequence.len(), 1);
        assert_eq!(rle_sequence.0[0].count, 100_000);
    }
}
