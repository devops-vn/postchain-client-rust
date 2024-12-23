use crate::encoding::gtv;
use crate::utils::hasher::gtv_hash;

use super::{keypair, params::Operation};

use hex::FromHex;

pub enum TransactionErrors {
    BRIDIsEmpty
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

    pub fn sign(&mut self, private_key: &[u8; 64]) -> Result<(), secp256k1::Error> {
        let bytes = Vec::from_hex(private_key).unwrap();
        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&bytes[0..32]);

        let public_key = keypair::get_public_key(&private_key_bytes)?;

        if self.signers.is_none() {
                self.signers = Some(Vec::new());
        }

        if let Some(ref mut signers) = self.signers.as_mut() {
            signers.push(public_key.to_vec());
        }

        let to_draw_gtx = gtv::to_draw_gtx(self);

        let digest = gtv_hash(to_draw_gtx);

        let digest_array: [u8; 32] = digest.try_into().expect("Invalid digest to sign");
        let signature = keypair::sign(&digest_array, &private_key_bytes)?;

        if self.signatures.is_none() {
            self.signatures = Some(Vec::new());
        }

        if let Some(ref mut signatures) = self.signatures.as_mut() {
            signatures.push(signature.to_vec());
        }

        Ok(())
    }
}