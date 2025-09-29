use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use crypto::crypto::{decrypt_data, derive_key, encrypt_data, restore_key};

#[derive(Clone, Debug)]
pub struct Wallet {
    secret: [u8; 32],
    address: [u8; 33],
}

impl Wallet {
    pub fn new() -> Self {
        let secret = libsecp256k1::SecretKey::random(&mut rand::rngs::OsRng);
        let public = libsecp256k1::PublicKey::from_secret_key(&secret);
        Wallet {
            secret: secret.serialize(),
            address: public.serialize_compressed(),
        }
    }

    pub fn secret(&self) -> [u8; 32] {
        self.secret.clone()
    }

    pub fn address(&self) -> String {
        hex::encode(self.address)
    }

    pub fn read(dir: &str, address: &str, password: &[u8]) -> Result<Self, std::io::Error> {
        let path = format!("{}/{}", dir, address);
        let mut file = File::open(path)?;
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        let mut data = Vec::new();
        file.read_exact(&mut salt)?;
        file.read_exact(&mut nonce)?;
        file.read_to_end(&mut data)?;
        let key = restore_key(&salt, password)?;
        let secret = decrypt_data(&key, data.as_slice(), &nonce)?;
        let secret: [u8; 32] = secret[..].try_into().unwrap();
        Self::from_secret(secret)
    }

    pub fn from_secret(secret: [u8; 32]) -> Result<Self, std::io::Error> {
        let secret = libsecp256k1::SecretKey::parse(&secret)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        let public = libsecp256k1::PublicKey::from_secret_key(&secret);
        Ok(Wallet {
            secret: secret.serialize(),
            address: public.serialize_compressed(),
        })
    }

    pub fn write(&self, dir: &str, password: &[u8]) -> Result<(), std::io::Error> {
        fs::create_dir_all(dir)?;
        let (salt, key) = derive_key(password)?;
        let (data, nonce) = encrypt_data(&key, &self.secret)?;
        let mut file = OpenOptions::new().write(true).create(true).open(format!(
            "{}/{}",
            dir,
            hex::encode(self.address)
        ))?;
        file.write_all(&salt)?;
        file.write_all(&nonce)?;
        file.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn sign(&self, data: &[u8; 32]) -> Result<String, std::io::Error> {
        let secret = libsecp256k1::SecretKey::parse(&self.secret).unwrap();
        let (signature, _) = libsecp256k1::sign(&libsecp256k1::Message::parse(data), &secret);
        Ok(hex::encode(signature.serialize()))
    }
}
