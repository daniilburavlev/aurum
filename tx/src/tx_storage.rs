use crate::tx::Tx;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::sync::Arc;

const LATEST_HASH_KEY: &str = "tx.latest";

pub struct TxStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl TxStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        let db = Arc::clone(db);
        Self { db }
    }

    pub fn find_latest_hash(&self) -> Result<Option<String>, std::io::Error> {
        if let Some(value) = self
            .db
            .get(LATEST_HASH_KEY)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            Ok(Some(String::from_utf8(value).unwrap()))
        } else {
            Ok(None)
        }
    }

    fn save_latest_hash(&self, hash: String) -> Result<(), std::io::Error> {
        self.db
            .put(LATEST_HASH_KEY, hash.as_bytes())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    pub fn save(&self, txs: &Vec<Tx>, block: u64) -> Result<(), std::io::Error> {
        for tx in txs {
            self.save_without_idx(tx)?;
            self.add_to_txs_index(tx.from(), tx.hash_str())?;
            self.add_to_txs_index(tx.to(), tx.hash_str())?;
        }
        self.save_block_idx(txs, block)?;
        if let Some(latest) = txs.last() {
            self.save_latest_hash(latest.hash_str())?;
        }
        Ok(())
    }

    fn add_to_txs_index(&self, wallet: String, tx_hash: String) -> Result<(), std::io::Error> {
        let mut txs = self.find_wallet_txs_hashes(wallet.clone())?;
        txs.push(tx_hash);
        let data = serde_json::to_vec(&txs)?;
        let key = self.build_key(&wallet);
        self.db
            .put(key, data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn save_block_idx(&self, txs: &Vec<Tx>, block: u64) -> Result<(), std::io::Error> {
        let hashes = txs.iter().map(|tx| tx.hash_str()).collect::<Vec<String>>();
        let data = serde_json::to_vec(&hashes)?;
        let key = self.build_key(&block.to_string());
        self.db
            .put(key, &data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn find_hashes_by_block_idx(&self, idx: String) -> Result<Vec<String>, std::io::Error> {
        let key = self.build_key(&idx);
        if let Some(hashes) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            Ok(serde_json::from_slice(&hashes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
        } else {
            Ok(Vec::new())
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

    pub fn find_wallet_txs(&self, wallet: String) -> Result<Vec<Tx>, std::io::Error> {
        let hashes = self.find_wallet_txs_hashes(wallet)?;
        let mut txs = Vec::new();
        for hash in hashes {
            if let Some(tx) = self.find_by_hash(hash)? {
                txs.push(tx);
            }
        }
        Ok(txs)
    }

    fn find_wallet_txs_hashes(&self, wallet: String) -> Result<Vec<String>, std::io::Error> {
        let key = self.build_key(&wallet);
        match self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        {
            Some(txs) => Ok(serde_json::from_slice(&txs)?),
            None => Ok(Vec::new()),
        }
    }

    pub fn find_by_block_idx(&self, idx: u64) -> Result<Vec<Tx>, std::io::Error> {
        let hashes = self.find_hashes_by_block_idx(idx.to_string())?;
        let mut txs = Vec::new();
        for hash in hashes {
            if let Some(tx) = self.find_by_hash(hash)? {
                txs.push(tx);
            }
        }
        Ok(txs)
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
