use crate::utils::params::OperationParams;
use crate::encoding::gtv;

pub enum TransactionErrors {
    BRIDIsEmpty
}

pub struct Transaction<'a> {
    pub blockchain_rid: &'a str,
    pub operations: Vec<(&'a str, OperationParams<'a>)>,
    pub signers: Vec<&'a [u8]>,
    pub signatures: Vec<&'a [u8]>
}

impl<'a> Transaction<'a> {
    pub fn new(blockchain_rid: &'a str,
        operations: Vec<(&'a str, OperationParams<'a>)>,
        signers: Vec<&'a [u8]>,
        signatures: Vec<&'a [u8]>) -> Self {
        Self {
            blockchain_rid,
            operations,
            signers,
            signatures
        }
    }

    pub fn gvt_hex_encoded(&self) -> String{
        let gtv_e = gtv::encode_tx(self);
        hex::encode(gtv_e)
    }

    pub fn get_signatures_from_tx(tx: &str) -> Vec<&u8> {
        let txe = hex::decode(tx).unwrap();

        let result = gtv::decode_tx(&txe);
        let _data = result.unwrap();
        
        vec![]
    }

    pub fn decode(_tx: &str) -> Self {
        let blockchain_rid = "";
        let operations = vec![];
        let signers = vec![];
        let signatures = vec![];
        Self {
            blockchain_rid,
            operations,
            signers,
            signatures
        }       
    }

}

#[test]
fn transaction_simple1_test() {
    let blockchain_rid = "7DE352DAD68988E3AA7B019E9FB178F58E4A4CB5FA0CED3C97277008F33700C8";
    let operations = vec![
        ("setInteger", OperationParams::Integer(10302))
    ];
    let signers: Vec<&[u8]> = vec![];
    let signatures: Vec<&[u8]> = vec![];
    let tx = Transaction::new(blockchain_rid,
        operations,
        signers,
        signatures);
    
    assert_eq!(tx.gvt_hex_encoded(),
    "a5523050a54a3048a12204207de352dad68988e3aa7b019e9fb178f58e4a4cb5fa0ced3c97277008f33700c8a51e301ca51a3018a20c0c0a736574496e7465676572a5083006a3040202283ea5023000a5023000".to_string());
}

#[test]
fn transaction_simple2_test() {
    let signature1 = hex::decode("73dbd6ee4a37e5bb0142cbcdf3bd1de5d5d7fe1ae58eb67caa30d4aea393c51260dd9f6ca1e2cd614f9c9196dc7c64e1c7719a0efbb0ba70dbb986343cfe9b56").unwrap();

    let blockchain_rid = "7DE352DAD68988E3AA7B019E9FB178F58E4A4CB5FA0CED3C97277008F33700C8";
    let operations = vec![
        ("setArray", OperationParams::Array(vec![
            OperationParams::Integer(1234), OperationParams::Text("bla bla")
        ]))
    ];
    let signers: Vec<&[u8]> = vec![];
    let signatures: Vec<&[u8]> = vec![
        &signature1
    ];
    let tx = Transaction::new(blockchain_rid,
        operations,
        signers,
        signatures);

    assert_eq!(tx.gvt_hex_encoded(),
    "a581a43081a1a5573055a12204207de352dad68988e3aa7b019e9fb178f58e4a4cb5fa0ced3c97277008f33700c8a52b3029a5273025a20a0c087365744172726179a5173015a5133011a304020204d2a2090c07626c6120626c61a5023000a5463044a142044073dbd6ee4a37e5bb0142cbcdf3bd1de5d5d7fe1ae58eb67caa30d4aea393c51260dd9f6ca1e2cd614f9c9196dc7c64e1c7719a0efbb0ba70dbb986343cfe9b56".to_string());
}