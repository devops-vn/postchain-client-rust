use crate::encoding::gtv;
use crate::utils::hasher::gtv_hash;
use super::params::Operation;
use secp256k1::{PublicKey, Secp256k1, SecretKey, Message, ecdsa::Signature};

use hex::FromHex;

#[derive(Debug, PartialEq)]
pub enum TransactionStatus {
    REJECTED,
    CONFIRMED,
    WAITING,
    UNKNOWN
}

pub struct Transaction<'a> {
    pub blockchain_rid: Vec<u8>,
    pub operations: Option<Vec<Operation<'a>>>,
    pub signers: Option<Vec<Vec<u8>>>,
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

    pub fn gvt_hex_encoded(&self) -> String{
        let gtv_e = gtv::encode_tx(self);
        let hex_encode = hex::encode(gtv_e);
        hex_encode
    }

    pub fn tx_rid(&self) -> Vec<u8> {
        let to_draw_gtx = gtv::to_draw_gtx(self);
        gtv_hash(to_draw_gtx)
    }

    pub fn tx_rid_hex(&self) -> String {
        hex::encode(self.tx_rid())
    }

    pub fn sign_from_raw_priv_key(&mut self, private_key: &str) -> Result<(), secp256k1::Error> {
        let bytes = hex::decode(hex::encode(private_key)).unwrap();
        let mut priv_key_array = [0u8; 64];
        priv_key_array.copy_from_slice(&bytes);
        self.sign(&priv_key_array)
    }

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

    pub fn multi_sign(&mut self, private_keys: &[&[u8; 64]]) -> Result<(), secp256k1::Error> {
        for private_key in private_keys {
            self.sign(private_key)?;
        }
        Ok(())
    }
}

fn sign(digest: &[u8; 32], private_key: &[u8; 32]) -> Result<[u8; 64], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let message = Message::from_digest(*digest);
    let signature: Signature = secp.sign_ecdsa(&message, &secret_key);
    let serialized_signature = signature.serialize_compact();
    Ok(serialized_signature)
}

fn get_public_key(private_key: &[u8; 32]) -> Result<[u8; 33], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key).serialize();
    Ok(public_key)
}
