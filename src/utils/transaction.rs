use crate::encoding::gtv;

use super::params::Operation;

pub enum TransactionErrors {
    BRIDIsEmpty
}

pub struct Transaction<'a> {
    pub blockchain_rid: &'a str,
    pub operations: Option<Vec<Operation<'a>>>,
    pub signers: Option<Vec<&'a [u8]>>,
    pub signatures: Option<Vec<&'a [u8]>>
}

impl<'a> Default for Transaction<'a> {
    fn default() -> Self {
        Self {
            blockchain_rid: "",
            operations: None,   
            signers: None,      
            signatures: None    
        }
    }
}

impl<'a> Transaction<'a> {
    pub fn new(blockchain_rid: &'a str,
        operations: Option<Vec<Operation<'a>>>,
        signers: Option<Vec<&'a [u8]>>,
        signatures: Option<Vec<&'a [u8]>>) -> Self {
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
}