//! Transaction handling and digital signature functionality.
//! 
//! This module provides functionality for creating, managing, and signing blockchain
//! transactions. It supports single and multi-signature transactions using ECDSA
//! with the secp256k1 curve.
//! 
//! # Features
//! - Transaction creation and management
//! - Transaction ID generation
//! - Single and multi-signature support
//! - GTV (Generic Tree Value) encoding
//! 
//! # Example
//! ```
//! use crate::utils::transaction::{Transaction, TransactionStatus};
//! 
//! // Create a new transaction
//! let mut tx = Transaction::new(
//!     vec![1, 2, 3],  // blockchain RID
//!     Some(vec![]),   // operations
//!     None,           // signers
//!     None            // signatures
//! );
//! 
//! // Sign the transaction
//! let private_key = [0u8; 64];  // Replace with actual private key
//! tx.sign(&private_key).expect("Failed to sign transaction");
//! ```

use crate::encoding::gtv;
use crate::utils::hasher::gtv_hash;
use super::operation::Operation;
use secp256k1::{PublicKey, Secp256k1, SecretKey, Message, ecdsa::Signature};
use hex::FromHex;

/// Represents the current status of a transaction in the blockchain.
#[derive(Debug, PartialEq)]
pub enum TransactionStatus {
    /// Transaction was rejected by the blockchain
    REJECTED,
    /// Transaction has been confirmed and included in a block
    CONFIRMED,
    /// Transaction is waiting to be included in a block
    WAITING,
    /// Transaction status is unknown
    UNKNOWN
}

/// Represents a blockchain transaction with operations and signatures.
/// 
/// A transaction contains a list of operations to be executed, along with
/// the necessary signatures to authorize these operations. It supports
/// both single and multi-signature scenarios.
pub struct Transaction<'a> {
    /// Unique identifier of the blockchain this transaction belongs to
    pub blockchain_rid: Vec<u8>,
    /// List of operations to be executed in this transaction
    pub operations: Option<Vec<Operation<'a>>>,
    /// List of public keys of the signers
    pub signers: Option<Vec<Vec<u8>>>,
    /// List of signatures corresponding to the signers
    pub signatures: Option<Vec<Vec<u8>>>
}

impl<'a> Default for Transaction<'a> {
    fn default() -> Self {
        Self {
            blockchain_rid: vec![],
            operations: None,   
            signers: None,      
            signatures: None    
        }
    }
}

impl<'a> Transaction<'a> {
    /// Creates a new transaction with the specified parameters.
    /// 
    /// # Arguments
    /// * `blockchain_rid` - Unique identifier of the blockchain
    /// * `operations` - Optional list of operations to be executed
    /// * `signers` - Optional list of public keys of the signers
    /// * `signatures` - Optional list of signatures
    /// 
    /// # Returns
    /// A new Transaction instance
    pub fn new(blockchain_rid: Vec<u8>,
        operations: Option<Vec<Operation<'a>>>,
        signers: Option<Vec<Vec<u8>>>,
        signatures: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            blockchain_rid,
            operations,
            signers,
            signatures
        }
    }

    /// Returns the hex-encoded GTV (Generic Tree Value) representation of the transaction.
    /// 
    /// This method encodes the transaction into GTV format and returns it as a
    /// hexadecimal string.
    /// 
    /// # Returns
    /// Hex-encoded string of the GTV-encoded transaction
    pub fn gvt_hex_encoded(&self) -> String {
        let gtv_e = gtv::encode_tx(self);
        let hex_encode = hex::encode(gtv_e);
        hex_encode
    }

    /// Computes the unique identifier (RID) of this transaction.
    /// 
    /// The transaction RID is computed by hashing the GTV representation
    /// of the transaction using the GTX hash function.
    /// 
    /// # Returns
    /// Vector of bytes containing the transaction RID
    pub fn tx_rid(&self) -> Vec<u8> {
        let to_draw_gtx = gtv::to_draw_gtx(self);
        gtv_hash(to_draw_gtx)
    }

    /// Returns the hex-encoded transaction RID.
    /// 
    /// This is a convenience method that returns the transaction RID
    /// as a hexadecimal string.
    /// 
    /// # Returns
    /// Hex-encoded string of the transaction RID
    pub fn tx_rid_hex(&self) -> String {
        hex::encode(self.tx_rid())
    }

    /// Signs the transaction using a raw private key string.
    /// 
    /// # Arguments
    /// * `private_key` - Private key as a string
    /// 
    /// # Returns
    /// Result indicating success or a secp256k1 error
    /// 
    /// # Errors
    /// Returns an error if the private key is invalid or signing fails
    pub fn sign_from_raw_priv_key(&mut self, private_key: &str) -> Result<(), secp256k1::Error> {
        let bytes = hex::decode(hex::encode(private_key)).unwrap();
        let mut priv_key_array = [0u8; 64];
        priv_key_array.copy_from_slice(&bytes);
        self.sign(&priv_key_array)
    }

    /// Signs the transaction using a private key.
    /// 
    /// This method:
    /// 1. Derives the public key from the private key
    /// 2. Adds the public key to the signers list
    /// 3. Signs the transaction RID
    /// 4. Adds the signature to the signatures list
    /// 
    /// # Arguments
    /// * `private_key` - 64-byte private key
    /// 
    /// # Returns
    /// Result indicating success or a secp256k1 error
    /// 
    /// # Errors
    /// Returns an error if the private key is invalid or signing fails
    pub fn sign(&mut self, private_key: &[u8; 64]) -> Result<(), secp256k1::Error> {
        let bytes = Vec::from_hex(private_key).unwrap();
        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&bytes[0..32]);

        let public_key = get_public_key(&private_key_bytes)?;

        if self.signers.is_none() {
                self.signers = Some(Vec::new());
        }

        if let Some(ref mut signers) = self.signers.as_mut() {
            signers.push(public_key.to_vec());
        }

        let digest = self.tx_rid();

        let digest_array: [u8; 32] = digest.clone().try_into().expect("Invalid digest to sign");
        let signature = sign(&digest_array, &private_key_bytes)?;

        if self.signatures.is_none() {
            self.signatures = Some(Vec::new());
        }

        if let Some(ref mut signatures) = self.signatures.as_mut() {
            signatures.push(signature.to_vec());
        }

        Ok(())
    }

    /// Signs the transaction with multiple private keys.
    /// 
    /// This method iteratively signs the transaction with each provided
    /// private key, enabling multi-signature transactions.
    /// 
    /// # Arguments
    /// * `private_keys` - Slice of 64-byte private keys
    /// 
    /// # Returns
    /// Result indicating success or a secp256k1 error
    /// 
    /// # Errors
    /// Returns an error if any private key is invalid or signing fails
    pub fn multi_sign(&mut self, private_keys: &[&[u8; 64]]) -> Result<(), secp256k1::Error> {
        for private_key in private_keys {
            self.sign(private_key)?;
        }
        Ok(())
    }
}

/// Signs a message digest using ECDSA with secp256k1.
/// 
/// # Arguments
/// * `digest` - 32-byte message digest to sign
/// * `private_key` - 32-byte private key
/// 
/// # Returns
/// Result containing the 64-byte signature or a secp256k1 error
/// 
/// # Errors
/// Returns an error if the private key is invalid or signing fails
fn sign(digest: &[u8; 32], private_key: &[u8; 32]) -> Result<[u8; 64], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let message = Message::from_digest(*digest);
    let signature: Signature = secp.sign_ecdsa(&message, &secret_key);
    let serialized_signature = signature.serialize_compact();
    Ok(serialized_signature)
}

/// Derives a public key from a private key using secp256k1.
/// 
/// # Arguments
/// * `private_key` - 32-byte private key
/// 
/// # Returns
/// Result containing the 33-byte compressed public key or a secp256k1 error
/// 
/// # Errors
/// Returns an error if the private key is invalid
fn get_public_key(private_key: &[u8; 32]) -> Result<[u8; 33], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key).serialize();
    Ok(public_key)
}
