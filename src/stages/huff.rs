use crate::mtf::{MtfIndex, MtfTransform};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

type SymbolIndex = usize;
type SymbolCount = usize;

type FrequencyMap = HashMap<usize, usize>;

trait FrequencyMapping {
    fn build(mtf: &MtfTransform) -> Self;
}

impl FrequencyMapping for FrequencyMap {
    /// Builds a frequency map from the given Move-to-Front (MTF) transform.
    ///
    /// This method constructs a `FrequencyMap` by iterating over the indices
    /// of the provided `MtfTransform` and counting the occurrences of each symbol.
    /// The symbols are derived from the MTF indices as follows:
    /// - `MtfIndex::RunA` is mapped to `0`.
    /// - `MtfIndex::RunB` is mapped to `1`.
    /// - `MtfIndex::Val(i)` is mapped to `i + 1`.
    ///
    /// Additionally, an End-Of-Block (EOB) symbol is inserted into the map with
    /// a frequency of `1`. The EOB symbol is calculated as the number of symbols
    /// in the stack of the MTF transform plus `1`.
    ///
    /// # Parameters
    /// - `mtf`: A reference to an `MtfTransform` instance from which the frequency
    ///   map will be built.
    ///
    /// # Returns
    /// A `FrequencyMap` containing the frequency of each symbol derived from the
    /// MTF transform, including the EOB symbol.
    fn build(mtf: &MtfTransform) -> Self {
        let mut freq_map = FrequencyMap::new();
        // insert RUNA and RUNB into map
        freq_map.insert(0, 0);
        freq_map.insert(1, 0);

        let symbols = mtf.indices().iter().map(|idx| match idx {
            MtfIndex::RunA => 0usize,
            MtfIndex::RunB => 1,
            MtfIndex::Val(i) => (*i as usize) + 1,
        });
        for sym in symbols {
            *freq_map.entry(sym as usize).or_insert(0) += 1;
        }
        // insert EOB into map
        let eob = mtf.num_stack().max(1) + 1;
        freq_map.insert(eob, 1);

        freq_map
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    freq: usize,
    symbol: Option<usize>,
}

impl Node {
    fn new_leaf(freq: usize, symbol: Option<usize>) -> Node {
        Node {
            left: None,
            right: None,
            freq,
            symbol,
        }
    }

    fn new_branch(left: Node, right: Node) -> Node {
        let freq = left.freq + right.freq;
        Node {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            freq,
            symbol: None,
        }
    }
}

pub fn encode(mtf_encode: MtfTransform) -> Result<Node> {
    let freq_map = FrequencyMap::build(&mtf_encode);
    let mut freq_list: Vec<Node> = Vec::with_capacity(freq_map.len());
    for (data, freq) in freq_map {
        freq_list.push(Node::new_leaf(freq, Some(data)));
    }

    //Sort the Vector
    freq_list.sort_by(|a, b| b.symbol.cmp(&a.symbol));
    freq_list.sort_by(|a, b| b.freq.cmp(&a.freq));

    while freq_list.len() != 1 {
        let left_node = freq_list.pop().ok_or(anyhow!("Missing Left Node"))?;
        let right_node = freq_list.pop().ok_or(anyhow!("Missing Right Node"))?;
        let new_node = Node::new_branch(left_node, right_node);
        freq_list.push(new_node);
        freq_list.sort_by(|a, b| b.freq.cmp(&a.freq));
    }
    freq_list.pop().ok_or(anyhow!("Missing Root Node"))
}

#[cfg(test)]
mod tests {
    use crate::mtf::t;

    use super::*;
    use test_case::test_case;

    /// utility method to easily construct MtfTransform structs in tests
    fn get_mtf(indices: Vec<usize>, stack: Vec<u8>) -> MtfTransform {
        MtfTransform {
            indices: indices
                .iter()
                .map(|&i| match i {
                    t::RUNA => MtfIndex::RunA,
                    t::RUNB => MtfIndex::RunB,
                    v => MtfIndex::Val(v as u8),
                })
                .collect(),
            stack,
        }
    }

    #[test_case(vec![], vec![] => HashMap::from([(0, 0), (1, 0), (2, 1)]); "empty")]
    #[test_case(vec![t::RUNA, t::RUNA, t::RUNB], vec![0] => HashMap::from([(0, 2), (1, 1), (2, 1)]); "one run")]
    #[test_case(vec![1, 2, 3, t::RUNA, t::RUNA, t::RUNB], vec![1, 10, 100, 42] => HashMap::from([
        (0, 2), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)]); "run at end")]
    fn test_freq_map(indices: Vec<usize>, stack: Vec<u8>) -> FrequencyMap {
        FrequencyMap::build(&get_mtf(indices, stack))
    }

    #[test_case(MtfTransform::empty() => Node {
        left: Some(Box::new(Node {
            left: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: Some(0) })),
            right: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: Some(1) })), freq: 0, symbol: None })),
        right: Some(Box::new(Node { left: None, right: None, freq: 1, symbol: Some(2) })), freq: 1, symbol: None }; "empty")]
    fn test_encode(mtf: MtfTransform) -> Node {
        encode(mtf).unwrap()
    }

    #[test_case(vec![], vec![] => 1; "empty")]
    #[test_case(vec![t::RUNA, t::RUNA, t::RUNB], vec![0] => 4; "one run")]
    #[test_case(vec![1, 2, 3, t::RUNA, t::RUNA, t::RUNB], vec![1, 10, 100, 42] => 7; "run at end")]
    fn test_freqs_in_tree(indices: Vec<usize>, stack: Vec<u8>) -> usize {
        encode(get_mtf(indices, stack)).unwrap().freq
    }
    #[test_case(0, None => Node { left: None, right: None, freq: 0, symbol: None }; "empty")]
    #[test_case(1337, Some(42) => Node { left: None, right: None, freq: 1337, symbol: Some(42) }; "lotsa 42s")]
    #[test_case(1337, None => Node { left: None, right: None, freq: 1337, symbol: None }; "whole lotta nuthin")]
    fn test_new_leaf(freq: usize, symbol: Option<usize>) -> Node {
        Node::new_leaf(freq, symbol)
    }

    #[test_case(0, None, 0, None => Node {
        left: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: None })),
        right: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: None })),
        freq: 0,
        symbol: None }; "empty")]
    #[test_case(1312, Some(42), 25, Some(23) => Node {
        left: Some(Box::new(Node { left: None, right: None, freq: 1312, symbol: Some(42) })),
        right: Some(Box::new(Node { left: None, right: None, freq: 25, symbol: Some(23) })),
        freq: 1337,
        symbol: None }; "long")]
    fn test_new_branch(
        freq_left: usize,
        symbol_left: Option<usize>,
        freq_right: usize,
        symbol_right: Option<usize>,
    ) -> Node {
        Node::new_branch(
            Node::new_leaf(freq_left, symbol_left),
            Node::new_leaf(freq_right, symbol_right),
        )
    }

    /// Requirement: "NumSyms is computed as NumStack - 1 + 3 , where NumStack is the number of
    /// symbols in the stack from the MTF stage."
    #[test_case(vec![], vec![] => 3; "empty")]
    #[test_case(vec![t::RUNA, t::RUNA, t::RUNB], vec![0] => 3; "one run")]
    #[test_case(vec![1, 2, 3, t::RUNA, t::RUNA, t::RUNB], vec![1, 10, 100, 42] => 6; "run at end")]
    #[test_case(vec![t::RUNA, t::RUNB], vec![ 97 ] => 3; "one symbol")]
    #[test_case(vec![t::RUNA, 1, 1, 1, 1, 1], vec![ 97, 98 ] => 4; "run at beginning")]
    #[test_case(vec![t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2, t::RUNA, 1, 2], vec![ 97, 98, 99 ] => 5; "back and forth")]
    #[test_case(vec![t::RUNA, 1, 2, t::RUNA, t::RUNA], vec![97, 98, 99] => 5; "runas at end")]
    #[test_case(vec![1, t::RUNA, 4, 2, 3, t::RUNA, t::RUNB, 1, 4, 2, t::RUNB, 3, 4, 5, t::RUNB, t::RUNA, t::RUNA, 2, 1], vec![97, 98, 101, 102, 121, 122] => 8; "bbyaeeeeeeafeeeybzzzzzzzzzyz")]
    fn test_num_syms(indices: Vec<usize>, stack: Vec<u8>) -> usize {
        FrequencyMap::build(&get_mtf(indices, stack)).len()
    }
}
