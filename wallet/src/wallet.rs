use crypto::crypto::{decrypt_data, derive_key, encrypt_data, restore_key};
use libp2p::identity::secp256k1;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Wallet {
    keypair: secp256k1::Keypair,
}

impl Wallet {
    pub fn new() -> Self {
        let secret = secp256k1::SecretKey::generate();
        let keypair = secp256k1::Keypair::from(secret);
        Self { keypair }
    }

    pub fn keypair(&self) -> secp256k1::Keypair {
        self.keypair.clone()
    }

    pub fn address(&self) -> [u8; 33] {
        self.keypair.public().to_bytes()
    }

    pub fn address_str(&self) -> String {
        bs58::encode(self.address()).into_string()
    }

    pub fn secret(&self) -> [u8; 32] {
        self.keypair.secret().to_bytes()
    }

    pub fn secret_str(&self) -> String {
        bs58::encode(self.secret()).into_string()
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
        let secret: [u8; 32] = secret.try_into().unwrap();
        Self::from_secret(secret)
    }

    pub fn from_secret(secret: [u8; 32]) -> Result<Self, std::io::Error> {
        let secret = secp256k1::SecretKey::try_from_bytes(secret)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        Ok(Wallet {
            keypair: secp256k1::Keypair::from(secret),
        })
    }

    pub fn from_secret_str(secret: String) -> Result<Self, std::io::Error> {
        let secret = bs58::decode(secret)
            .into_vec()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        let secret: [u8; 32] = secret.try_into().unwrap();
        Self::from_secret(secret)
    }

    pub fn write(&self, dir: &str, password: &[u8]) -> Result<(), std::io::Error> {
        fs::create_dir_all(dir)?;
        let (salt, key) = derive_key(password)?;
        let (data, nonce) = encrypt_data(&key, &self.secret())?;
        let mut file = OpenOptions::new().write(true).create(true).open(format!(
            "{}/{}",
            dir,
            self.address_str(),
        ))?;
        file.write_all(&salt)?;
        file.write_all(&nonce)?;
        file.write_all(data.as_slice())?;
        Ok(())
    }

    pub fn sign(&self, data: &[u8; 32]) -> Result<String, std::io::Error> {
        Ok(bs58::encode(self.keypair.secret().sign(data)).into_string())
    }

    pub fn verify(&self, data: &[u8; 32], signature: String) -> bool {
        let signature = bs58::encode(signature).into_vec();
        self.keypair.public().verify(data, &signature)
    }
}
