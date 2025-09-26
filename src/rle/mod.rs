#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RleSequence(Vec<u8>);

impl RleSequence {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn sequence(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn encode(data: &[u8]) -> Self {
        // worst case is x1.25 if data consists solely of sequences of four (e.g. b"aaaabbbbaaaabbbb")
        let mut sequence = Vec::with_capacity(data.len() * 125 / 100);
        let chunks = data.chunk_by(|a, b| a == b);

        for chunk in chunks {
            let value = chunk[0];
            let mut remaining_length = chunk.len();

            while remaining_length > 0 {
                let run_length = remaining_length.min(255);

                if run_length < 4 {
                    sequence.extend(std::iter::repeat(value).take(run_length));
                } else {
                    sequence.extend(std::iter::repeat(value).take(4));
                    sequence.push((run_length - 4) as u8);
                }

                remaining_length -= run_length;
            }
        }
        Self(sequence)
    }

    pub fn decode(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut iter = self.0.iter();

        let mut consecutive_count = 0;
        let mut last_byte = None;

        while let Some(&byte) = iter.next() {
            data.push(byte);
            if Some(byte) == last_byte {
                consecutive_count += 1;
            } else {
                consecutive_count = 1;
                last_byte = Some(byte);
            }

            if consecutive_count == 4 {
                if let Some(&run_length) = iter.next() {
                    data.extend(std::iter::repeat(byte).take(run_length as usize));
                }
                consecutive_count = 0;
                last_byte = None;
            }
        }

        data
    }
}

impl From<&[u8]> for RleSequence {
    fn from(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}

impl From<Vec<u8>> for RleSequence {
    fn from(data: Vec<u8>) -> Self {
        Self(data)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(&[] => RleSequence(Vec::<u8>::new()); "empty")]
    #[test_case(b"aaaaa" => RleSequence(vec![b'a', b'a', b'a', b'a', 1]); "five same bytes")]
    #[test_case(b"a" => RleSequence(vec![b'a']); "one byte")]
    #[test_case(b"aaaab" => RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b']); "four same one different")]
    #[test_case(b"aaaa" => RleSequence(vec![b'a', b'a', b'a', b'a', 0]); "shortest worst case")]
    #[test_case(b"aaaabbbbaaaa" => RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b', b'b', b'b', b'b', 0, b'a', b'a', b'a', b'a', 0]); "longer worst case")]
    #[test_case(b"aaaabcdefg" => RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b', b'c', b'd', b'e', b'f', b'g']); "repeat at beginning")]
    #[test_case(b"aaaaabcdefg" => RleSequence(vec![b'a', b'a', b'a', b'a', 1, b'b', b'c', b'd', b'e', b'f', b'g']); "repeat plus one at beginning")]
    #[test_case(b"xyzaaaabc" => RleSequence(vec![b'x', b'y', b'z', b'a', b'a', b'a', b'a', 0, b'b', b'c']); "repeat in the middle")]
    #[test_case(b"xyzaaaaabc" => RleSequence(vec![b'x', b'y', b'z', b'a', b'a', b'a', b'a', 1, b'b', b'c']); "repeat plus one in the middle")]
    #[test_case(b"abcdddd" => RleSequence(vec![b'a', b'b', b'c', b'd', b'd', b'd', b'd', 0]); "repeat at end")]
    #[test_case(b"abcddddd" => RleSequence(vec![b'a', b'b', b'c', b'd', b'd', b'd', b'd', 1]); "repeat plus one at end")]
    #[test_case(&[b'a'; 255] => RleSequence(vec![b'a', b'a', b'a', b'a', 251]); "long run")]
    #[test_case(&[b'a'; 256] => RleSequence(vec![b'a', b'a', b'a', b'a', 251, b'a']); "overlong run")]
    fn test_rle_encode(data: &[u8]) -> RleSequence {
        RleSequence::encode(data)
    }

    #[test_case(RleSequence(Vec::<u8>::new()) => Vec::<u8>::new(); "empty")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 1]) => b"aaaaa".to_vec(); "five same bytes")]
    #[test_case(RleSequence(vec![b'a']) => b"a".to_vec(); "one byte")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b']) => b"aaaab".to_vec(); "four same one different")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 0]) => b"aaaa".to_vec(); "shortest worst case")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b', b'b', b'b', b'b', 0, b'a', b'a', b'a', b'a', 0]) => b"aaaabbbbaaaa".to_vec(); "longer worst case")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 0, b'b', b'c', b'd', b'e', b'f', b'g']) => b"aaaabcdefg".to_vec(); "repeat at beginning")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 1, b'b', b'c', b'd', b'e', b'f', b'g']) => b"aaaaabcdefg".to_vec(); "repeat plus one at beginning")]
    #[test_case(RleSequence(vec![b'x', b'y', b'z', b'a', b'a', b'a', b'a', 0, b'b', b'c']) => b"xyzaaaabc".to_vec(); "repeat in the middle")]
    #[test_case(RleSequence(vec![b'x', b'y', b'z', b'a', b'a', b'a', b'a', 1, b'b', b'c']) => b"xyzaaaaabc".to_vec(); "repeat plus one in the middle")]
    #[test_case(RleSequence(vec![b'a', b'b', b'c', b'd', b'd', b'd', b'd', 0]) => b"abcdddd".to_vec(); "repeat at end")]
    #[test_case(RleSequence(vec![b'a', b'b', b'c', b'd', b'd', b'd', b'd', 1]) => b"abcddddd".to_vec(); "repeat plus one at end")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 251]) => [b'a'; 255].to_vec(); "long run")]
    #[test_case(RleSequence(vec![b'a', b'a', b'a', b'a', 251, b'a']) => [b'a'; 256].to_vec(); "overlong run")]
    fn test_rle_decode(seq: RleSequence) -> Vec<u8> {
        seq.decode()
    }

    #[test_case(&[]; "empty")]
    #[test_case(b"aaaaa"; "five same bytes")]
    #[test_case(b"a"; "one byte")]
    #[test_case(b"aaaab"; "four same one different")]
    #[test_case(b"aaaa"; "shortest worst case")]
    #[test_case(b"aaaabbbbaaaa"; "longer worst case")]
    #[test_case(b"aaaabcdefg"; "repeat at beginning")]
    #[test_case(b"aaaaabcdefg"; "repeat plus one at beginning")]
    #[test_case(b"xyzaaaabc"; "repeat in the middle")]
    #[test_case(b"xyzaaaaabc"; "repeat plus one in the middle")]
    #[test_case(b"abcdddd"; "repeat at end")]
    #[test_case(b"abcddddd"; "repeat plus one at end")]
    #[test_case(&[b'a'; 255]; "long run")]
    #[test_case(&[b'a'; 256]; "overlong run")]
    fn test_roundtrip(data: &[u8]) {
        assert_eq!(data, RleSequence::encode(data).decode());
    }
}
