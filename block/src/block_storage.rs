use crate::block::Block;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::str::FromStr;
use std::sync::Arc;

pub struct BlockStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl BlockStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub fn save(&self, block: &Block) -> Result<(), std::io::Error> {
        let mut block = block.clone();
        block.txs = None;
        let json = serde_json::to_vec(&block)?;
        let key = self.build_key(&block.idx.to_string());
        self.db
            .put(key, json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let key = self.build_key(&block.hash_str());
        self.db
            .put(key, block.idx.to_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.db
            .put("block.latest", block.idx.to_string())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn find_by_idx(&self, idx: u64) -> Result<Option<Block>, std::io::Error> {
        let key = self.build_key(&idx.to_string());
        if let Some(json) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        {
            let block: Block = serde_json::from_slice(&json)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_hash(&self, hash: String) -> Result<Option<Block>, std::io::Error> {
        let key = self.build_key(&hash);
        if let Some(json) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        {
            let idx: u64 = serde_json::from_slice(&json)?;
            return self.find_by_idx(idx);
        }
        Ok(None)
    }

    pub fn find_latest(&self) -> Result<Option<Block>, std::io::Error> {
        if let Some(idx) = self
            .db
            .get("block.latest")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        {
            let block =
                self.find_by_idx(u64::from_str(&String::from_utf8(idx).unwrap()).unwrap())?;
            Ok(block)
        } else {
            Ok(None)
        }
    }

    fn build_key(&self, value: &str) -> String {
        format!("block.{}", value)
    }
}
