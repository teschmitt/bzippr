use crate::mtf::{MtfIndex, MtfTransform};
use std::collections::HashMap;

type SymbolIndex = usize;
type SymbolCount = usize;

type FrequencyMap = HashMap<SymbolIndex, SymbolCount>;

const MAX_HUFFMAN_LEN: usize = 20;

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
        let mut freq_map: HashMap<SymbolIndex, SymbolCount> = FrequencyMap::new();
        // insert RUNA and RUNB into map
        freq_map.insert(0, 0);
        freq_map.insert(1, 0);

        let symbols = mtf.indices().iter().map(|idx| match idx {
            MtfIndex::RunA => 0,
            MtfIndex::RunB => 1,
            MtfIndex::Val(i) => (*i as SymbolIndex) + 1,
        });
        for sym in symbols {
            *freq_map.entry(sym as SymbolIndex).or_insert(0) += 1;
        }
        // insert EOB into map
        let eob = (mtf.num_stack().max(1) + 1) as SymbolIndex;
        freq_map.insert(eob, 1);

        freq_map
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
    freq: SymbolCount,
    symbol: Option<SymbolIndex>,
}

impl Node {
    fn new_leaf(freq: SymbolCount, symbol: Option<SymbolIndex>) -> Self {
        Self {
            left: None,
            right: None,
            freq,
            symbol,
        }
    }

    fn new_branch(left: Self, right: Self) -> Self {
        let freq = left.freq + right.freq;
        Self {
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            freq,
            symbol: None,
        }
    }

    /// Calculates the depth of the current node in the tree.
    ///
    /// The depth is determined by traversing the tree recursively and finding
    /// the longest path from the current node to a leaf node. A leaf node is
    /// defined as a node with no children (`left` and `right` are `None`).
    ///
    /// # Returns
    ///
    /// * `usize` - The depth of the node, where a leaf node has a depth of 1.
    fn get_depth(&self) -> usize {
        match (&self.left, &self.right) {
            (None, None) => 1,
            (None, Some(right)) => 1 + right.get_depth(),
            (Some(left), None) => 1 + left.get_depth(),
            (Some(left), Some(right)) => 1 + left.get_depth().max(right.get_depth()),
        }
    }
}

type SymbolCode = Vec<bool>;
type CodeTable = HashMap<Option<SymbolIndex>, SymbolCode>;

struct HuffmanEncoder {
    root: Option<Node>,
    code_table: CodeTable,
}

impl HuffmanEncoder {
    pub fn new(mtf_encode: &MtfTransform) -> Self {
        let Some(mut root) = Self::build_tree(mtf_encode) else {
            return Self::empty();
        };
        while root.get_depth() > MAX_HUFFMAN_LEN {
            Self::rebalance(&mut root);
        }
        let mut code_table = CodeTable::new();
        let mut cur_sym_code = SymbolCode::new();
        Self::get_codes(&root, &mut cur_sym_code, &mut code_table);
        Self {
            root: Some(root),
            code_table: code_table.clone(),
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            root: None,
            code_table: HashMap::new(),
        }
    }

    fn build_tree(mtf_encode: &MtfTransform) -> Option<Node> {
        let freq_map = FrequencyMap::build(mtf_encode);
        let mut freq_list: Vec<Node> = Vec::with_capacity(freq_map.len());
        for (data, freq) in freq_map {
            freq_list.push(Node::new_leaf(freq, Some(data)));
        }

        // sort in ascending order
        freq_list.sort_by(|a, b| b.freq.cmp(&a.freq));

        while freq_list.len() != 1 {
            // TODO: Lmax of bzip2 is 20, so the tree cannot be deeper. Check for this constraint
            let left_node = freq_list.pop().unwrap(); // TODO: Error handling
            let right_node = freq_list.pop().unwrap(); // TODO: Error handling
            let new_node = Node::new_branch(left_node, right_node);
            freq_list.push(new_node);
            freq_list.sort_by(|a, b| b.freq.cmp(&a.freq));
        }

        freq_list.pop()
    }

    fn rebalance(_node: &mut Node) {
        // TODO: yeah abracadabra rebalance this tree!
        todo!()
    }

    /// Build code table for Huffman tree by traversing the tree with a DFS
    fn get_codes(node: &Node, current_symbol_code: &mut SymbolCode, code_table: &mut CodeTable) {
        match (&node.left, &node.right) {
            (None, None) => {
                // leaf, so save the code table entry
                code_table.insert(node.symbol, current_symbol_code.clone());
            }
            (None, Some(right)) => {
                current_symbol_code.push(true);
                Self::get_codes(&right, current_symbol_code, code_table);
            }
            (Some(left), None) => {
                current_symbol_code.push(false);
                Self::get_codes(&left, current_symbol_code, code_table);
            }
            (Some(left), Some(right)) => {
                let mut current_symbol_code_left = current_symbol_code.clone();
                current_symbol_code_left.push(false); // for the left branch
                current_symbol_code.push(true); // for the right branch
                Self::get_codes(&left, &mut current_symbol_code_left, code_table);
                Self::get_codes(&right, current_symbol_code, code_table);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mtf::t;

    use super::*;
    use test_case::test_case;

    /// utility method to easily construct MtfTransform structs in tests
    fn get_mtf(indices: Vec<SymbolIndex>, stack: Vec<u8>) -> MtfTransform {
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
    fn test_freq_map(indices: Vec<SymbolIndex>, stack: Vec<u8>) -> FrequencyMap {
        FrequencyMap::build(&get_mtf(indices, stack))
    }

    #[test_case(MtfTransform::empty() => Node {
        left: Some(Box::new(Node {
            left: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: Some(0) })),
            right: Some(Box::new(Node { left: None, right: None, freq: 0, symbol: Some(1) })), freq: 0, symbol: None })),
        right: Some(Box::new(Node { left: None, right: None, freq: 1, symbol: Some(2) })), freq: 1, symbol: None }; "empty")]
    fn test_encode(mtf: MtfTransform) -> Node {
        HuffmanEncoder::new(&mtf).root.unwrap()
    }

    #[test_case(vec![], vec![] => 1; "empty")]
    #[test_case(vec![t::RUNA, t::RUNA, t::RUNB], vec![0] => 4; "one run")]
    #[test_case(vec![1, 2, 3, t::RUNA, t::RUNA, t::RUNB], vec![1, 10, 100, 42] => 7; "run at end")]
    fn test_freqs_in_tree(indices: Vec<SymbolIndex>, stack: Vec<u8>) -> usize {
        HuffmanEncoder::new(&get_mtf(indices, stack))
            .root
            .unwrap()
            .freq
    }
    #[test_case(0, None => Node { left: None, right: None, freq: 0, symbol: None }; "empty")]
    #[test_case(1337, Some(42) => Node { left: None, right: None, freq: 1337, symbol: Some(42) }; "lotsa 42s")]
    #[test_case(1337, None => Node { left: None, right: None, freq: 1337, symbol: None }; "whole lotta nuthin")]
    fn test_new_leaf(freq: SymbolCount, symbol: Option<SymbolIndex>) -> Node {
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
        freq_left: SymbolCount,
        symbol_left: Option<SymbolIndex>,
        freq_right: SymbolCount,
        symbol_right: Option<SymbolIndex>,
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
    fn test_num_syms(indices: Vec<SymbolIndex>, stack: Vec<u8>) -> usize {
        FrequencyMap::build(&get_mtf(indices, stack)).len()
    }
}
