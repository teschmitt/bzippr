#[derive(Debug, PartialEq, Eq)]
pub struct BwtEncoded {
    pub data: Vec<u8>,
    pub original_index: usize,
}

impl BwtEncoded {
    pub(crate) fn empty() -> Self {
        BwtEncoded {
            data: Vec::new(),
            original_index: 0,
        }
    }
}

pub(crate) fn bwt_encode(data: &[u8]) -> BwtEncoded {
    BwtEncoded::empty()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(b"BANANA" => BwtEncoded { data: vec![b'B', b'N', b'N', b'A', b'A', b'A'], original_index: 6 }; "banana")]
    fn test_bwt_encode(data: &[u8]) -> BwtEncoded {
        bwt_encode(data)
    }
}
