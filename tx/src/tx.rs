use bigdecimal::BigDecimal;
use crypto::crypto::verify_signature;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::cmp::Ordering;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use wallet::wallet::Wallet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: String,
    pub block: Option<u64>,
}

impl Ord for Tx {
    fn cmp(&self, other: &Self) -> Ordering {
        let order = self.timestamp.cmp(&other.timestamp);
        match order {
            Ordering::Equal => {
                self.hash.cmp(&other.hash)
            },
            _ => order,
        }
    }
}

impl PartialOrd for Tx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Tx {
    pub fn new(
        wallet: &Wallet,
        to: String,
        amount: String,
        nonce: u64,
    ) -> Result<Self, std::io::Error> {
        BigDecimal::from_str(amount.as_str())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        let mut tx = Self {
            hash: "".to_string(),
            from: wallet.address(),
            to,
            amount,
            nonce,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: "".to_string(),
            block: None,
        };
        let signature = wallet.sign(&tx.hash())?;
        tx.signature = signature;
        tx.hash = tx.hash_str();
        Ok(tx)
    }

    pub fn from(&self) -> String {
        self.from.to_string()
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn to(&self) -> String {
        self.to.to_string()
    }

    pub fn amount(&self) -> BigDecimal {
        BigDecimal::from_str(self.amount.as_str()).unwrap()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        let hash = self.hash();
        hex::encode(&hash)
    }

    pub fn valid(&self) -> bool {
        if BigDecimal::from_str(self.amount.as_str()).is_err() {
            return false;
        }
        match hex::decode(&self.from) {
            Ok(public_key) => verify_signature(public_key, &self.signature, &self.hash()),
            Err(_) => false,
        }
    }
}
