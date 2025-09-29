use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::sync::Arc;
pub struct NonceStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}
impl NonceStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        let db = Arc::clone(db);
        Self { db }
    }

    pub fn save(&self, wallet: String, nonce: u64) -> Result<(), std::io::Error> {
        let key = self.build_key(&wallet);
        let data = serde_json::to_vec(&nonce)?;
        self.db
            .put(key, data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    pub fn get(&self, wallet: String) -> Result<u64, std::io::Error> {
        let key = self.build_key(&wallet);
        if let Some(nonce) = self
            .db
            .get(key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        {
            Ok(serde_json::from_slice(&nonce)?)
        } else {
            Ok(0)
        }
    }

    fn build_key(&self, value: &str) -> String {
        format!("nonce.{}", value)
    }
}
