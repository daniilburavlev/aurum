use crate::tx_data::TxData;
use common::bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx {
    pub data: TxData,
    pub prev_hash: String,
    pub block: u64,
    pub hash: String,
}

impl Tx {
    pub fn from_tx(data: TxData, prev_hash: String, block: u64) -> Self {
        let mut tx = Self {
            prev_hash,
            data,
            block,
            hash: String::default(),
        };
        tx.hash = tx.hash_str();
        tx
    }

    pub fn from(&self) -> String {
        self.data.from.to_string()
    }

    pub fn nonce(&self) -> u64 {
        self.data.nonce
    }

    pub fn to(&self) -> String {
        self.data.to.to_string()
    }

    pub fn amount(&self) -> BigDecimal {
        self.data.amount.clone()
    }

    pub fn prev_hash(&self) -> String {
        self.prev_hash.clone()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(self.block.to_be_bytes());
        hasher.update(self.data.hash());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        let hash = self.hash();
        bs58::encode(&hash).into_string()
    }

    pub fn valid(&self) -> bool {
        if self.block == 0 {
            return true;
        }
        self.hash == self.hash_str() && self.data.valid()
    }
}
