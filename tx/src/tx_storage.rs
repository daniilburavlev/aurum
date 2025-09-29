use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;
use crate::tx::Tx;

const NO_BLOCK_IDX: &str = "empty";

pub struct TxStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl TxStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        let db = Arc::clone(db);
        Self { db }
    }

    pub fn save(&self, tx: &Tx) -> Result<(), std::io::Error> {
        self.save_without_idx(tx)?;
        self.add_to_txs_index(tx.from(), tx.hash_str())?;
        self.add_to_txs_index(tx.to(), tx.hash_str())?;
        self.add_to_block_idx(tx)?;
        Ok(())
    }

    fn add_to_txs_index(&self, wallet: String, tx_hash: String) -> Result<(), std::io::Error> {
        let mut txs = self.find_wallet_txs_hashes(wallet.clone())?;
        txs.insert(tx_hash);
        let data = serde_json::to_vec(&txs)?;
        let key = self.build_key(&wallet);
        self.db
            .put(key, data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn add_to_block_idx(&self, tx: &Tx) -> Result<(), std::io::Error> {
        let idx = if let Some(idx) = tx.block {
            idx.to_string()
        } else {
            String::from(NO_BLOCK_IDX)
        };
        let mut hashes = self.find_hashes_by_block_idx(idx.clone())?;
        hashes.insert(tx.hash_str());
        let data = serde_json::to_vec(&hashes)?;
        let key = self.build_key(&idx);
        self.db
            .put(key, &data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn find_hashes_by_block_idx(&self, idx: String) -> Result<HashSet<String>, std::io::Error> {
        let key = self.build_key(&idx);
        if let Some(hashes) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            Ok(serde_json::from_slice(&hashes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
        } else {
            Ok(HashSet::new())
        }
    }

    pub fn find_by_hash(&self, hash: String) -> Result<Option<Tx>, std::io::Error> {
        let key = self.build_key(&hash);
        if let Some(data) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            let tx: Tx = serde_json::from_slice(&data)?;
            Ok(Some(tx))
        } else {
            Ok(None)
        }
    }

    pub fn find_wallet_txs(&self, wallet: String) -> Result<BTreeSet<Tx>, std::io::Error> {
        let hashes = self.find_wallet_txs_hashes(wallet)?;
        let mut txs = BTreeSet::new();
        for hash in hashes {
            if let Some(tx) = self.find_by_hash(hash)? {
                txs.insert(tx);
            }
        }
        Ok(txs)
    }

    fn find_wallet_txs_hashes(&self, wallet: String) -> Result<HashSet<String>, std::io::Error> {
        let key = self.build_key(&wallet);
        match self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            Some(txs) => Ok(serde_json::from_slice(&txs)?),
            None => Ok(HashSet::new()),
        }
    }

    pub fn find_pending(&self) -> Result<BTreeSet<Tx>, std::io::Error> {
        let hashes = self.find_hashes_by_block_idx(String::from(NO_BLOCK_IDX))?;
        let mut txs = BTreeSet::new();
        for hash in hashes {
            if let Some(tx) = self.find_by_hash(hash)? {
                txs.insert(tx);
            }
        }
        Ok(txs)
    }

    pub fn find_by_block_idx(&self, idx: u64) -> Result<BTreeSet<Tx>, std::io::Error> {
        let hashes = self.find_hashes_by_block_idx(idx.to_string())?;
        let mut txs = BTreeSet::new();
        for hash in hashes {
            if let Some(tx) = self.find_by_hash(hash)? {
                txs.insert(tx);
            }
        }
        Ok(txs)
    }

    pub fn update_pending(&self, txs: &BTreeSet<Tx>, idx: u64) -> Result<(), std::io::Error> {
        let mut hashes = self.find_hashes_by_block_idx(String::from(NO_BLOCK_IDX))?;
        let mut new_idx = HashSet::new();
        for tx in txs {
            let mut tx = tx.clone();
            tx.block = Some(idx);
            hashes.remove(&tx.hash_str());
            new_idx.insert(tx.hash_str());
            self.save_without_idx(&tx)?;
        }
        let hashes = serde_json::to_string(&hashes)?;
        self.db
            .put(self.build_key(NO_BLOCK_IDX), hashes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let new_idx = serde_json::to_string(&new_idx)?;
        self.db
            .put(self.build_key(&idx.to_string()), new_idx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn build_key(&self, value: &str) -> String {
        format!("tx.{}", value)
    }

    fn save_without_idx(&self, tx: &Tx) -> Result<(), std::io::Error> {
        let json = serde_json::to_vec(&tx)?;
        self.db
            .put(self.build_key(&tx.hash_str()), json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }
}
