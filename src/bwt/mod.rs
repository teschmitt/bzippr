use anyhow::{Ok, Result, anyhow};

#[derive(Debug, PartialEq, Eq)]
pub struct BwtEncoded {
    data: Vec<u8>,
    original_index: usize,
}

impl TryFrom<&[u8]> for BwtEncoded {
    type Error = anyhow::Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Ok(BwtEncoded::empty());
        }
        let mut shifts = get_shifts(data)?;
        let original_index = sort_table(&mut shifts);
        let last_column: Vec<u8> = shifts
            .iter()
            .map(|shift| shift.last().copied().ok_or(anyhow!("Shift is empty")))
            .collect::<Result<Vec<u8>>>()?;
        Ok(BwtEncoded::new(last_column, original_index))
    }
}

impl TryInto<Vec<u8>> for BwtEncoded {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        let data_length = self.len();
        if data_length == 0 {
            return Ok(Vec::new());
        }
        let mut data_table: Vec<Vec<u8>> = Vec::with_capacity(data_length);

        for _ in 0..data_length {
            data_table.push(vec![0; data_length]);
        }

        for col in (0..self.len()).rev() {
            for row in 0..self.len() {
                data_table[row][col] = self.try_get(row)?;
            }
            sort_table(&mut data_table);
        }

        Ok(data_table
            .get(self.original_index)
            .ok_or(anyhow!(
                "Original index out of bounds: {}",
                self.original_index
            ))?
            .clone())
    }
}

impl BwtEncoded {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn new(data: Vec<u8>, original_index: usize) -> Self {
        BwtEncoded {
            data,
            original_index,
        }
    }

    pub fn empty() -> Self {
        BwtEncoded {
            data: Vec::new(),
            original_index: 0,
        }
    }

    fn try_get(&self, index: usize) -> Result<u8> {
        self.data
            .get(index)
            .copied()
            .ok_or(anyhow!("Index out of bounds: {}", index))
    }
}

fn get_from_index(data: &[u8], index: usize) -> Result<Vec<u8>> {
    let data_length = data.len();
    let mut result = Vec::with_capacity(data_length);
    let mut current_index = index;
    for _ in 0..data_length {
        result.push(
            *data
                .get(current_index)
                .ok_or(anyhow!("Index out of bounds: {}", current_index))?,
        );
        current_index = (current_index + 1) % data_length;
    }
    Ok(result)
}

fn get_shifts(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let data_length = data.len();
    if data_length == 0 {
        return Ok(vec![Vec::new()]);
    }
    let mut ret = Vec::with_capacity(data_length);
    for idx in 0..data_length {
        ret.push(get_from_index(data, idx)?);
    }
    Ok(ret)
}

fn sort_table(data_table: &mut Vec<Vec<u8>>) -> usize {
    if data_table.is_empty() || data_table.len() == 1 {
        return 0;
    }
    let orig = &data_table[0].clone();
    data_table.sort_unstable();
    data_table
        .iter()
        .position(|shift| shift.eq(orig))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(b"aba" => BwtEncoded { data: b"baa".to_vec(), original_index: 1 }; "aab")]
    #[test_case(b"zbcba" => BwtEncoded { data: b"bczba".to_vec(), original_index: 4 }; "zbcba")]
    #[test_case(b"a" => BwtEncoded { data: b"a".to_vec(), original_index: 0 }; "single byte")]
    #[test_case(b"aaa" => BwtEncoded { data: b"aaa".to_vec(), original_index: 0 }; "three identical bytes")]
    #[test_case(b"" => BwtEncoded { data: b"".to_vec(), original_index: 0 }; "empty")]
    fn test_bwt_encode(data: &[u8]) -> BwtEncoded {
        data.try_into().unwrap()
    }

    #[test_case(b"ANABAN", 3 => b"BANANA".to_vec(); "banana")]
    #[test_case(b"AB", 0 => b"AB".to_vec(); "index 0")]
    #[test_case(b"AB", 1 => b"BA".to_vec(); "index -1")]
    #[test_case(b"", 1 => b"".to_vec(); "empty")]
    fn test_get_from_index_success(data: &[u8], index: usize) -> Vec<u8> {
        get_from_index(data, index).unwrap()
    }

    #[test_case(b"abc" => vec![b"abc".to_vec(), b"bca".to_vec(), b"cab".to_vec()]; "three bytes")]
    #[test_case(b"ab" => vec![b"ab".to_vec(), b"ba".to_vec()]; "two bytes")]
    #[test_case(b"a" => vec![vec![b'a']]; "one byte")]
    #[test_case(b"" => vec![Vec::<u8>::new()]; "empty")]
    fn test_get_shifts_success(data: &[u8]) -> Vec<Vec<u8>> {
        get_shifts(data).unwrap()
    }

    #[test_case(vec![], vec![] => 0; "empty")]
    #[test_case(vec![b"sadfiuasdiufasiudfnasdf".to_vec()], vec![b"sadfiuasdiufasiudfnasdf".to_vec()] => 0; "one element")]
    #[test_case(vec![vec![100, 1], vec![1, 100]], vec![vec![1, 100], vec![100, 1]] => 1; "switch entries")]
    #[test_case(vec![vec![1, 100], vec![100, 1]], vec![vec![1, 100], vec![100, 1]] => 0; "already sorted")]
    fn test_sort_table(mut input: Vec<Vec<u8>>, expected: Vec<Vec<u8>>) -> usize {
        let idx = sort_table(&mut input);
        assert_eq!(input, expected);
        idx
    }

    #[test_case(BwtEncoded { data: b"baa".to_vec(), original_index: 1 }, b"aba".to_vec(); "three bytes")]
    #[test_case(BwtEncoded { data: b"bczba".to_vec(), original_index: 4 }, b"zbcba".to_vec(); "five bytes")]
    #[test_case(BwtEncoded { data: b"a".to_vec(), original_index: 0 }, b"a".to_vec(); "single byte")]
    #[test_case(BwtEncoded { data: b"aaa".to_vec(), original_index: 0 }, b"aaa".to_vec(); "three identical bytes")]
    #[test_case(BwtEncoded { data: b"".to_vec(), original_index: 0 }, b"".to_vec(); "empty")]
    fn test_bwt_decode(encoded: BwtEncoded, expected: Vec<u8>) {
        let decoded: Vec<u8> = encoded.try_into().unwrap();
        assert_eq!(decoded, expected);
    }

    const LARGE_DATA: &str = "\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aaaaaaaaaa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567...................78901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa45678.......abcdefgh....8901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa456789......abcdefgh.....901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890.....abcdefgh......01234567890zzzzzzzzzz12345678901234567890\
12345678901234567890123aaaaaaaaaa45678901....abcdefgh.......1234567890zzzzzzzzzz12345678901234567890\
123456789012345678901234567890123456789012...abcdefgh........234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123..YZABCDEFGHIJKLM..34567890zzzzzzzzzz12345678901234567890\
12345678901234567890123456789012345678901234...BCDEFGHIJKLMNO..4567890zzzzzzzzzz12345678901234567890\
123456789012345678901234567890123456789012345...DEFGHIJKLMNOPQ..567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456...................67890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
123456789012345678901234567890123456789012345678\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aaaaaaaaaa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890123456789012345678901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567...................78901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa45678.......abcdefgh....8901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa456789......abcdefgh.....901234567890123456789012345678901234567890\
12345678901234567890123aa000000aa4567890.....abcdefgh......01234567890zzzzzzzzzz12345678901234567890\
12345678901234567890123aaaaaaaaaa45678901....abcdefgh.......1234567890zzzzzzzzzz12345678901234567890\
123456789012345678901234567890123456789012...abcdefgh........234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123..YZABCDEFGHIJKLM..34567890zzzzzzzzzz12345678901234567890\
12345678901234567890123456789012345678901234...BCDEFGHIJKLMNO..4567890zzzzzzzzzz12345678901234567890\
123456789012345678901234567890123456789012345...DEFGHIJKLMNOPQ..567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456...................67890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890zzzzzzzzzz12345678901234567890\
1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\
123456789012345678901234567890123456789012345678";

    #[test_case(b"baa"; "three bytes")]
    #[test_case(b"bczba"; "five bytes")]
    #[test_case(b"a"; "single byte")]
    #[test_case(b"aaa"; "three identical bytes")]
    #[test_case("🚂⭐️🐝🤯".as_bytes(); "emojis")]
    #[test_case(LARGE_DATA.as_bytes(); "four kb")]
    #[test_case(b""; "empty")]
    fn test_roundtrip(data: &[u8]) {
        let encoded: BwtEncoded = data.try_into().unwrap();
        let decoded: Vec<u8> = encoded.try_into().unwrap();
        assert_eq!(decoded, data);
    }
}
