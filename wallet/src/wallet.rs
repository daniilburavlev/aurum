use crypto::crypto::{decrypt_data, derive_key, encrypt_data, restore_key};
use libp2p::identity::ecdsa;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Wallet {
    keypair: ecdsa::Keypair,
}

impl Wallet {
    pub fn new() -> Self {
        let secret = ecdsa::SecretKey::generate();
        let keypair = ecdsa::Keypair::from(secret);
        Self { keypair }
    }

    pub fn keypair(&self) -> ecdsa::Keypair {
        self.keypair.clone()
    }

    pub fn address(&self) -> Vec<u8> {
        self.keypair.public().to_bytes()
    }

    pub fn address_str(&self) -> String {
        bs58::encode(self.address()).into_string()
    }

    pub fn secret(&self) -> Vec<u8> {
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
        Self::from_secret(secret)
    }

    pub fn from_secret(secret: Vec<u8>) -> Result<Self, std::io::Error> {
        let secret = ecdsa::SecretKey::try_from_bytes(secret)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        Ok(Wallet {
            keypair: ecdsa::Keypair::from(secret),
        })
    }

    pub fn from_secret_str(secret: String) -> Result<Self, std::io::Error> {
        let secret = bs58::decode(secret)
            .into_vec()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
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
        Ok(bs58::encode(self.keypair.sign(data)).into_string())
    }
}
