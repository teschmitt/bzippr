#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RleSequence(Vec<u8>);

impl RleSequence {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn empty() -> Self {
        Self(vec![])
    }
}

impl From<&[u8]> for RleSequence {
    fn from(data: &[u8]) -> Self {
        // TODO: make this look nicer and more functional
        let mut idx_data = 0;
        let data_length = data.len();
        let mut sequence: Vec<u8> = Vec::with_capacity(data_length * 13 / 10);
        while idx_data < data_length {
            let current_value = data[idx_data];
            let mut look_ahead = 0;
            while idx_data + look_ahead < data_length
                && current_value == data[idx_data + look_ahead]
            {
                if look_ahead < 4 {
                    sequence.push(current_value);
                }
                if look_ahead == 255 {
                    break;
                }
                look_ahead += 1;
            }
            idx_data += look_ahead;
            if look_ahead >= 4 {
                sequence.push((look_ahead - 4) as u8);
            }
        }
        RleSequence(sequence)
    }
}

impl Into<Vec<u8>> for RleSequence {
    fn into(self) -> Vec<u8> {
        todo!()
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
    #[test_case(&[b'a'; 256] => RleSequence(vec![b'a', b'a', b'a', b'a', 251, b'a']); "loverlong run")]
    fn test_rle_encode(data: &[u8]) -> RleSequence {
        data.into()
    }
}
