use crypto::crypto::verify_signature;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tx::tx::Tx;
use wallet::wallet::Wallet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub idx: u64,
    pub validator: String,
    pub parent_hash: String,
    pub merkle_root: String,
    pub txs: Option<Vec<Tx>>,
    pub signature: String,
}

impl Block {
    pub fn new(
        wallet: &Wallet,
        idx: u64,
        parent_hash: String,
        txs: Vec<Tx>,
    ) -> Result<Self, std::io::Error> {
        let merkle_root = Self::merkle_root(&txs);
        let mut block = Self {
            idx,
            validator: wallet.address_str(),
            parent_hash,
            merkle_root: bs58::encode(merkle_root).into_string(),
            txs: Some(txs),
            signature: String::from(""),
        };
        block.signature = wallet.sign(&block.hash())?;
        Ok(block)
    }

    pub fn genesis(txs: Vec<Tx>) -> Self {
        let merkle_root = Block::merkle_root(&txs);
        let validator = [0u8; 33];
        let parent_hash = [0u8; 32];
        Block {
            idx: 0,
            validator: bs58::encode(validator).into_string(),
            parent_hash: bs58::encode(parent_hash).into_string(),
            merkle_root: bs58::encode(merkle_root).into_string(),
            txs: Some(txs),
            signature: String::from("GENESIS"),
        }
    }

    pub fn txs(&self) -> Option<Vec<Tx>> {
        self.txs.clone()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.idx.to_be_bytes());
        hasher.update(self.validator.as_bytes());
        hasher.update(self.parent_hash.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.finalize().into()
    }

    pub fn idx(&self) -> u64 {
        self.idx
    }

    pub fn hash_str(&self) -> String {
        bs58::encode(self.hash()).into_string()
    }

    pub fn last_event(&self) -> String {
        if let Some(txs) = self.txs.clone()
            && !txs.is_empty()
        {
            txs.last().unwrap().hash_str()
        } else {
            String::default()
        }
    }

    pub fn valid(&self) -> bool {
        let merkle_root = Block::merkle_root(self.txs.as_ref().unwrap());
        if self.merkle_root != bs58::encode(merkle_root).into_string() {
            return false;
        }
        for tx in self.txs.as_ref().unwrap() {
            if !tx.valid() {
                return false;
            }
        }
        match bs58::decode(self.validator.clone()).into_vec() {
            Ok(public_key) => {
                if let Ok(public_key) = public_key.try_into() {
                    verify_signature(public_key, &self.signature, &self.hash())
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn merkle_root(txs: &Vec<Tx>) -> [u8; 32] {
        let tx_hashes: Vec<[u8; 32]> = txs.clone().iter().map(|tx| tx.hash()).collect();
        let merkle_tree =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&tx_hashes);
        if let Some(merkle_root) = merkle_tree.root() {
            merkle_root
        } else {
            [0u8; 32]
        }
    }
}
