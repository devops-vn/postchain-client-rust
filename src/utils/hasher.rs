#![allow(warnings)]

//! Merkle tree-based hashing implementation for Generic Tree Value (GTV) data structures.
//! 
//! This module provides functionality to create Merkle trees from GTV data and compute
//! cryptographic hashes using SHA-256. The implementation supports different node types
//! including arrays, dictionaries, and leaf values, each with their own hash prefix.
//! 
//! # Hash Prefixes
//! - Leaf nodes: 1
//! - Internal nodes: 0
//! - Array nodes: 7
//! - Dictionary nodes: 8
//! 
//! # Example
//! ```
//! use std::collections::BTreeMap;
//! use crate::utils::operation::Params;
//! 
//! // Create a sample array
//! let data = Params::Array(vec![
//!     Params::Text("foo".to_string()),
//!     Params::Text("bar".to_string())
//! ]);
//! 
//! // Compute the GTV hash
//! let hash = gtv_hash(data);
//! ```

use sha2::{Sha256, Digest};
use crate::utils::operation::Params;
use crate::encoding::gtv::encode_value as gtv_encode_value;

/// Represents different types of nodes in the Merkle tree.
#[derive(Clone, PartialEq, Debug)]
enum NodeType {
    /// Internal node with two children
    Node,
    /// Leaf node containing actual data
    Leaf,
    /// Empty leaf node (used for padding)
    EmptyLeaf,
    /// Dictionary node containing key-value pairs
    DictNode,
    /// Array node containing ordered elements
    ArrayNode,
}

/// Represents a node in the binary Merkle tree.
/// 
/// Each node can be either an internal node with left and right children,
/// or a leaf node containing a value. The type of node is determined by
/// the `type_of_node` field.
#[derive(Clone, Debug)]
struct BinaryTreeNode {
    /// Left child of the node
    left: Option<Box<BinaryTreeNode>>,
    /// Right child of the node
    right: Option<Box<BinaryTreeNode>>,
    /// Value stored in the node (for leaf nodes)
    value: Option<Box<Params>>,
    /// Type of the node (internal, leaf, empty, etc.)
    type_of_node: NodeType
}

impl<'a> Default for BinaryTreeNode {
    fn default() -> Self {
        BinaryTreeNode {
            left: None,
            right: None,
            value: None,
            type_of_node: NodeType::EmptyLeaf,
        }
    }
}

impl<'a> BinaryTreeNode {
    /// Creates a new internal node with specified children, value, and type.
    /// 
    /// # Arguments
    /// * `left` - Left child node
    /// * `right` - Right child node
    /// * `value` - Optional value stored in the node
    /// * `type_of_node` - Type of the node
    fn new_node(left: Option<Box<BinaryTreeNode>>, right: Option<Box<BinaryTreeNode>>, value: Option<Box<Params>>, type_of_node: NodeType) -> Self {
        BinaryTreeNode {
            left, right, value, type_of_node
        }
    }

    /// Creates a new leaf node with an optional value.
    /// 
    /// # Arguments
    /// * `value` - Optional value to store in the leaf
    /// * `is_empty_leaf` - If true, creates an empty leaf node
    fn new_leaf(value: Option<Box<Params>>, is_empty_leaf: bool) -> Box<Self> {
        let type_of_node = match is_empty_leaf {
            true => NodeType::EmptyLeaf,
            false => NodeType::Leaf,
        };

        Box::new(BinaryTreeNode {
            value, type_of_node, ..Default::default()
        })
    }
}

/// Factory for creating binary Merkle trees from GTV data structures.
#[derive(Clone, Debug)]
struct BinaryTreeFactory;

impl<'a> BinaryTreeFactory {
    /// Processes a layer of nodes in the Merkle tree construction.
    /// 
    /// Combines pairs of nodes to create parent nodes until a single root is formed.
    /// 
    /// # Arguments
    /// * `leaves` - Vector of nodes to process
    /// 
    /// # Panics
    /// Panics if the input vector is empty
    fn process_layer(leaves: Vec<Box<BinaryTreeNode>>) -> Box<BinaryTreeNode> {
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

    /// Processes an array parameter into a Merkle tree node.
    /// 
    /// Creates a tree structure from an array of values, with array-specific
    /// hash prefixes for the nodes.
    /// 
    /// # Arguments
    /// * `params` - Box containing array parameters
    /// 
    /// # Panics
    /// Panics if the input is not an array parameter
    fn process_array_node(params: Box<Params>) -> Box<BinaryTreeNode> {
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

    /// Processes a dictionary parameter into a Merkle tree node.
    /// 
    /// Creates a tree structure from dictionary key-value pairs, with
    /// dictionary-specific hash prefixes for the nodes.
    /// 
    /// # Arguments
    /// * `params` - Box containing dictionary parameters
    /// 
    /// # Panics
    /// Panics if the input is not a dictionary parameter
    fn process_dict_node(params: Box<Params>) -> Box<BinaryTreeNode> {

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

    /// Creates a leaf node from a parameter value.
    /// 
    /// # Arguments
    /// * `params` - Box containing the parameter value
    fn process_leaf(params: Box<Params>) -> Box<BinaryTreeNode> {
        BinaryTreeNode::new_leaf(Some(params), false)
    }

    /// Builds a complete Merkle tree from a parameter value.
    /// 
    /// Recursively processes the input parameter based on its type
    /// (array, dictionary, or leaf value).
    /// 
    /// # Arguments
    /// * `params` - Box containing the parameter to process
    fn build_tree(params: Box<Params>) -> Box<BinaryTreeNode> {
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

/// Calculator for computing Merkle tree hashes using SHA-256.
struct MerkleHashCalculator;

const HASH_PREFIX_LEAF: u8 = 1;
const HASH_PREFIX_NODE: u8 = 0;
const HASH_PREFIX_NODE_ARRAY: u8 = 7;
const HASH_PREFIX_NODE_DICT: u8 = 8;

impl MerkleHashCalculator {
    /// Computes SHA-256 hash of input data.
    /// 
    /// # Arguments
    /// * `data` - Slice of bytes to hash
    fn sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Calculates hash for a leaf node.
    /// 
    /// Prepends the leaf prefix (1) to the encoded value before hashing.
    /// 
    /// # Arguments
    /// * `value` - Optional parameter value to hash
    fn calculate_leaf_hash(value: Option<Box<Params>>) -> Vec<u8> {
        let mut buffer = vec![HASH_PREFIX_LEAF];
        let encode_value = gtv_encode_value(&value.unwrap());
        buffer.extend_from_slice(&encode_value);
        Self::sha256(&buffer)
    }

    /// Calculates hash for an internal node.
    /// 
    /// Combines the node prefix and child hashes before hashing.
    /// 
    /// # Arguments
    /// * `has_prefix` - Node type prefix (0 for internal, 7 for array, 8 for dict)
    /// * `left` - Hash of left child
    /// * `right` - Hash of right child
    fn calculate_node_hash(has_prefix: u8, left: Vec<u8>, right: Vec<u8>) -> Vec<u8> {
        let mut buffer = vec![has_prefix];
        buffer.extend_from_slice(&left); 
        buffer.extend_from_slice(&right);
        Self::sha256(&buffer)
    }

    /// Recursively calculates the Merkle hash of a tree node.
    /// 
    /// # Arguments
    /// * `btn` - Root node of the tree or subtree
    fn calculate_merkle_hash(btn: Box<BinaryTreeNode>) -> Vec<u8>{
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

/// Computes the Merkle tree hash of a GTV (Generic Tree Value) parameter.
/// 
/// This function builds a Merkle tree from the input parameter and computes
/// its cryptographic hash using SHA-256. Different node types (array, dictionary,
/// leaf) use different hash prefixes to ensure unique representations.
/// 
/// # Arguments
/// * `value` - Parameter value to hash
/// 
/// # Returns
/// A vector of bytes containing the computed hash
/// 
/// # Example
/// ```
/// use std::collections::BTreeMap;
/// use crate::utils::operation::Params;
/// 
/// // Create a dictionary
/// let mut dict = BTreeMap::new();
/// dict.insert("key".to_string(), Params::Text("value".to_string()));
/// let data = Params::Dict(dict);
/// 
/// // Compute hash
/// let hash = gtv_hash(data);
/// ```
pub fn gtv_hash(value: Params) -> Vec<u8> {
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