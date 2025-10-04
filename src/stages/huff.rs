use crate::mtf::{MtfIndex, MtfTransform};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

type SymbolIndex = usize;
type SymbolCount = usize;

type FrequencyMap = HashMap<usize, usize>;

pub trait FrequencyMapping {
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
    /// i nthe stack of the MTF transform plus `1`.
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
        freq_map.insert(0, 1);
        freq_map.insert(1, 1);

        let symbols = mtf.indices().iter().map(|idx| match idx {
            MtfIndex::RunA => 0usize,
            MtfIndex::RunB => 1,
            MtfIndex::Val(i) => (*i as usize) + 1,
        });
        for sym in symbols {
            *freq_map.entry(sym as usize).or_insert(0) += 1;
        }
        // insert EOB into map
        let eob = mtf.num_stack() + 1;
        freq_map.insert(eob, 1);

        freq_map
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
    pub freq: usize,
    pub symbol: Option<usize>,
}

impl Node {
    fn new_leaf(freq: usize, value: Option<usize>) -> Node {
        Node {
            left: None,
            right: None,
            freq,
            symbol: value,
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
    use super::*;
    use test_case::test_case;

    #[test_case(MtfTransform::empty() => Node { left: Some(Box::new(Node { left: None, right: None, freq: 1, symbol: Some(0)})), right: Some(Box::new(Node { left: None, right: None, freq: 1, symbol: Some(1) })), freq: 2, symbol: None }; "empty")]
    fn test_frequency_map_build(mtf: MtfTransform) -> Node {
        encode(mtf).unwrap()
    }
}
