use secp256k1::{PublicKey, Secp256k1, SecretKey, Message, ecdsa::Signature};

pub fn sign(digest: &[u8; 32], private_key: &[u8; 32]) -> Result<[u8; 64], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let message = Message::from_digest(*digest);
    let signature: Signature = secp.sign_ecdsa(&message, &secret_key);
    let serialized_signature = signature.serialize_compact();
    Ok(serialized_signature)
}

pub fn get_public_key(private_key: &[u8; 32]) -> Result<[u8; 33], secp256k1::Error> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key).serialize();
    Ok(public_key)
}
