extern crate rand;

use rand::rngs::OsRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

#[derive(Debug)]
pub struct KeyPair {
    pub(crate) public_key: PublicKey,
    pub(crate) private_key: SecretKey,
}

impl KeyPair {
    pub(crate) fn generate_keypair() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let (private_key, public_key) = secp.generate_keypair(&mut rng);
        Self {
            public_key,
            private_key,
        }
    }

    pub(crate) fn from_private_key(private_key: &str) -> Self {
        let secp = Secp256k1::new();
        let bytes = hex::decode(private_key).expect("hex decode error");
        let private_key = SecretKey::from_slice(&bytes).expect("32 bytes, within curve order");

        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        Self {
            public_key,
            private_key,
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn print_keypair(&self) {
        let serialized_public_key = self.public_key.serialize();
        let hex_public_key = hex::encode(serialized_public_key);
        println!("pubkey={}", hex_public_key);
        println!("privkey={}", self.private_key.display_secret());
    }
}
