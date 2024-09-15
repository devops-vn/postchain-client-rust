use crate::utils::keypair;

use super::keypair::KeyPair;

#[derive(Debug)]
pub struct Signature {
    pub keypair: KeyPair,
}

impl Signature {
    pub fn new() -> Self {
        let keypair = keypair::KeyPair::generate_keypair();
        Self { keypair }
    }

    pub fn from_private_key(private_key: &str) -> Self {
        let keypair = keypair::KeyPair::from_private_key(private_key);
        Self { keypair }
    }

    pub fn public_key(&self) -> secp256k1::PublicKey {
        return self.keypair.public_key;
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        self.keypair.print_keypair();
    }
}
