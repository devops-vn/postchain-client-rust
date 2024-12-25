#![allow(warnings)]

use sha2::{Sha256, Digest};
use crate::utils::params::Params;
use crate::encoding::gtv::encode_value as gtv_encode_value;

#[derive(Clone, PartialEq, Debug)]
enum NodeType {
    Node,
    Leaf,
    EmptyLeaf,
    DictNode,
    ArrayNode,
}

#[derive(Clone, Debug)]
struct BinaryTreeNode<'a> {
    left: Option<Box<BinaryTreeNode<'a>>>,
    right: Option<Box<BinaryTreeNode<'a>>>,
    value: Option<Box<Params<'a>>>,
    type_of_node: NodeType
}

impl<'a> Default for BinaryTreeNode<'a> {
    fn default() -> Self {
        BinaryTreeNode {
            left: None,
            right: None,
            value: None,
            type_of_node: NodeType::EmptyLeaf,
        }
    }
}

impl<'a> BinaryTreeNode<'a> {
    fn new_node(left: Option<Box<BinaryTreeNode<'a>>>, right: Option<Box<BinaryTreeNode<'a>>>, value: Option<Box<Params<'a>>>, type_of_node: NodeType) -> Self {
        BinaryTreeNode {
            left, right, value, type_of_node
        }
    }

    fn new_leaf(value: Option<Box<Params<'a>>>, is_empty_leaf: bool) -> Box<Self> {
        let type_of_node = match is_empty_leaf {
            true => NodeType::EmptyLeaf,
            false => NodeType::Leaf,
        };

        Box::new(BinaryTreeNode {
            value, type_of_node, ..Default::default()
        })
    }
}

#[derive(Clone, Debug)]
struct BinaryTreeFactory;

impl<'a> BinaryTreeFactory {
    fn process_layer(leaves: Vec<Box<BinaryTreeNode<'_>>>) -> Box<BinaryTreeNode<'_>> {
        if leaves.is_empty() {
            panic!("Cannot work on empty arrays");
        }

        if leaves.len() == 1 {
            return leaves[0].clone();
        }

        let mut results= Vec::new();
        let mut i: usize = 0;

         while i < leaves.len() - 1 {
            let left = leaves[i].clone();
            let right = leaves[i + 1].clone();
            let node = BinaryTreeNode::new_node(Some(left), Some(right), None, NodeType::Node);
            results.push(Box::new(node));
            i += 2;
        }

        if i < leaves.len() {
            results.push(Box::new(*leaves[i].clone()));
        }

        return Self::process_layer(results);
    }

    fn process_array_node(params: Box<Params<'a>>) -> Box<BinaryTreeNode<'_>> {
        if params.clone().is_empty() {
            let left= BinaryTreeNode::new_leaf(None, true);
            let right= BinaryTreeNode::new_leaf(None, true);
            let value = Box::new(Params::Array(vec![]));
            let node = BinaryTreeNode::new_node(Some(left), Some(right), Some(value), NodeType::ArrayNode);
            return Box::new(node);
        }

        if let Params::Array(array_value) = *params {
            let mut leaves = Vec::new();

            for value in array_value.clone() {
                leaves.push(Self::build_tree(Box::new(value)));
            }

            let value = Box::new(Params::Array(array_value.clone()));
            let len = array_value.len();

            if leaves.len() == 1 {
                let left = leaves[0].clone();
                let right= BinaryTreeNode::new_leaf(None, true);
                let node = BinaryTreeNode::new_node(Some(left), Some(right), Some(value), NodeType::ArrayNode);
                return Box::new(node);
            }

            let tree_root = Self::process_layer(leaves);

            if tree_root.clone().type_of_node == NodeType::Node {
                let node = BinaryTreeNode::new_node(tree_root.clone().left, tree_root.right, Some(value), NodeType::ArrayNode);
                return Box::new(node);
            }

            let left = tree_root;
            let right= BinaryTreeNode::new_leaf(None, true);
            let node = BinaryTreeNode::new_node(Some(left), Some(right), Some(value), NodeType::ArrayNode);
            return Box::new(node);
        } else {
            panic!("Cannot process empty array!")
        }
    }

    fn process_dict_node(params: Box<Params<'a>>) -> Box<BinaryTreeNode<'_>> {

        if params.clone().is_empty() {
            let left= BinaryTreeNode::new_leaf(None, true);
            let right= BinaryTreeNode::new_leaf(None, true);
            let value = Box::new(Params::Array(vec![]));
            let node = BinaryTreeNode::new_node(Some(left), Some(right), Some(value), NodeType::DictNode);
            return Box::new(node);            
        }

        if let Params::Dict(dict_value) = *params {
            let mut leaves = Vec::new();

            let value = Box::new(Params::Dict(dict_value.clone()));
            let len = dict_value.len();

            for (key, value) in dict_value {
                leaves.push(BinaryTreeNode::new_leaf(Some(Box::new(Params::Text(key.to_string()))), false));
                leaves.push(Self::build_tree(Box::new(value)));
            }

            let tree_root = Self::process_layer(leaves);

            if tree_root.clone().type_of_node == NodeType::Node {
                let node = BinaryTreeNode::new_node(tree_root.clone().left, tree_root.right, Some(value), NodeType::DictNode);
                return Box::new(node);
            }

            let left = tree_root;
            let right= BinaryTreeNode::new_leaf(None, true);
            let node = BinaryTreeNode::new_node(Some(left), Some(right), Some(value), NodeType::DictNode);
            return Box::new(node);
        } else {
            panic!("Cannot process empty dict!")
        }
    }

    fn process_leaf(params: Box<Params<'a>>) -> Box<BinaryTreeNode<'_>> {
        BinaryTreeNode::new_leaf(Some(params), false)
    }

    fn build_tree(params: Box<Params<'a>>) -> Box<BinaryTreeNode<'_>> {
        match *params {
            Params::Array(_) =>
                Self::process_array_node(params),
            Params::Dict(_) =>
                Self::process_dict_node(params),
            _ =>
                Self::process_leaf(params)
        }
    }
}

struct MerkleHashCalculator;

const HASH_PREFIX_LEAF: u8 = 1;
const HASH_PREFIX_NODE: u8 = 0;
const HASH_PREFIX_NODE_ARRAY: u8 = 7;
const HASH_PREFIX_NODE_DICT: u8 = 8;

impl MerkleHashCalculator {
    fn sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    fn calculate_leaf_hash(value: Option<Box<Params<'_>>>) -> Vec<u8> {
        let mut buffer = vec![HASH_PREFIX_LEAF];
        let encode_value = gtv_encode_value(&value.unwrap());
        buffer.extend_from_slice(&encode_value);
        Self::sha256(&buffer)
    }

    fn calculate_node_hash(has_prefix: u8, left: Vec<u8>, right: Vec<u8>) -> Vec<u8> {
        let mut buffer = vec![has_prefix];
        buffer.extend_from_slice(&left); 
        buffer.extend_from_slice(&right);
        Self::sha256(&buffer)
    }

    fn calculate_merkle_hash(btn: Box<BinaryTreeNode<'_>>) -> Vec<u8>{
        if btn.type_of_node == NodeType::EmptyLeaf {
            return [0; 32].to_vec();
        }

        if btn.type_of_node == NodeType::Leaf {
            return Self::calculate_leaf_hash(btn.value);
        }

        let has_prefix = match btn.type_of_node {
            NodeType::ArrayNode => HASH_PREFIX_NODE_ARRAY,
            NodeType::DictNode => HASH_PREFIX_NODE_DICT,
            _ => HASH_PREFIX_NODE
        };

        return Self::calculate_node_hash(
            has_prefix,
            Self::calculate_merkle_hash(btn.left.unwrap()),
            Self::calculate_merkle_hash(btn.right.unwrap())
        )
    }
}

pub fn gtv_hash(value: Params<'_>) -> Vec<u8> {
    let tree = BinaryTreeFactory::build_tree(Box::new(value));
    let hash_value = MerkleHashCalculator::calculate_merkle_hash(tree);
    return hash_value;
}

#[test]
fn test_gtv_hash() {
    use std::collections::BTreeMap;

    let data1 = Params::Array(vec![
        Params::Text("foo".to_string()), Params::Array(vec![
            Params::Text("bar2".to_string()), Params::Text("bar2".to_string())
        ])
    ]);

    let mut data2_btree: BTreeMap<String, Params> = BTreeMap::new();
    data2_btree.insert("foo".to_string(), Params::Integer(-1));
    data2_btree.insert("foo1".to_string(), Params::Text("OK".to_string()));
    data2_btree.insert("bar".to_string(), Params::BigInteger(i128::MAX.into()));
    data2_btree.insert("bar1".to_string(), Params::BigInteger((1000000000000 as i128).into()));

    let data2 = Params::Dict(data2_btree);
    
    let result1 = gtv_hash(data1);
    let result2 = gtv_hash(data2);

    assert_eq!("6357d3200e0dfb1bce5f3eb789714842747b39810248f83dba6382c7e7020e20", hex::encode(result1));
    assert_eq!("9f3d80d08a942b86e20932ad74356703dba7ba78b792f2d6ad93201ab9a71bab", hex::encode(result2));
}