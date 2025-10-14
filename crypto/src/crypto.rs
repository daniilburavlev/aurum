use aes_gcm::aead::Aead;
use aes_gcm::KeyInit;
use argon2::PasswordHasher;
use rand::RngCore;

pub fn derive_key(password: &[u8]) -> Result<([u8; 16], [u8; 32]), std::io::Error> {
    let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2
        .hash_password(password, &salt)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
        .hash
        .unwrap();
    if password_hash.as_bytes().len() != 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid password hash",
        ));
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&password_hash.as_bytes());
    let mut salt_bytes = [0u8; 16];
    salt.decode_b64(&mut salt_bytes).unwrap();
    Ok((salt_bytes, key_bytes))
}

pub fn restore_key(salt: &[u8], password: &[u8]) -> Result<[u8; 32], std::io::Error> {
    let argon2 = argon2::Argon2::default();
    let salt = argon2::password_hash::SaltString::encode_b64(salt)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    let password_hash = argon2.hash_password(password, &salt).unwrap().hash.unwrap();
    if password_hash.as_bytes().len() != 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid password hash",
        ));
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(password_hash.as_bytes());
    Ok(key_bytes)
}

pub fn encrypt_data(key: &[u8], data: &[u8]) -> Result<(Vec<u8>, [u8; 12]), std::io::Error> {
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let cipher_data = cipher
        .encrypt(nonce, data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok((cipher_data, nonce_bytes))
}

pub fn decrypt_data(
    key: &[u8],
    data: &[u8],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let text = cipher
        .decrypt(nonce, data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(text)
}

pub fn verify_signature(public_key: [u8; 33], signature: &String, data: &[u8; 32]) -> bool {
    match libp2p::identity::secp256k1::PublicKey::try_from_bytes(&public_key) {
        Ok(public_key) => match bs58::decode(signature).into_vec() {
            Ok(signature) => public_key.verify(data, &signature),
            Err(_) => false,
        },
        Err(_) => false,
    }
}
