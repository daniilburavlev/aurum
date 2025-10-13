use crate::stake::Stake;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::sync::Arc;

const ALL_STAKE_WALLETS_KEY: &str = "stake.all_wallets";

pub struct StakeStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl StakeStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub fn save(&self, stake: &Stake) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_stake_wallets()?;
        wallets.insert(stake.wallet());
        let json = serde_json::to_vec(&stake)?;
        let key = Self::build_key(&stake.wallet);
        self.save_all_stake_wallets(&wallets)?;
        self.db.put(key, &json)?;
        Ok(())
    }

    pub fn save_all(&self, stakes: &Vec<Stake>) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_stake_wallets()?;
        for stake in stakes {
            wallets.insert(stake.wallet.clone());
            self.save(stake)?;
        }
        self.save_all_stake_wallets(&wallets)?;
        Ok(())
    }

    pub fn find(&self, wallet: String) -> Result<Option<Stake>, Box<dyn Error>> {
        let key = Self::build_key(&wallet);
        if let Some(json) = self.db.get(key)? {
            let value: Stake = serde_json::from_slice(&json)?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    pub fn find_all(&self, wallets: &HashSet<String>) -> Result<BTreeMap<String, Stake>, Box<dyn Error>> {
        let mut stakes = BTreeMap::new();
        for wallet in wallets {
            if let Some(stake) = self.find(wallet.to_owned())? {
                stakes.insert(wallet.to_owned(), stake);
            }
        }
        Ok(stakes)
    }

    pub fn load_all(&self) -> Result<BTreeMap<String, Stake>, Box<dyn Error>> {
        let wallets = self.all_stake_wallets()?;
        self.find_all(&wallets)
    }

    fn all_stake_wallets(&self) -> Result<HashSet<String>, Box<dyn Error>> {
        if let Some(value) = self.db.get(String::from(ALL_STAKE_WALLETS_KEY))? {
            Ok(serde_json::from_slice(&value)?)
        } else {
            Ok(HashSet::new())
        }
    }

    fn save_all_stake_wallets(&self, wallets: &HashSet<String>) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(&wallets)?;
        self.db.put(String::from(ALL_STAKE_WALLETS_KEY), json)?;
        Ok(())
    }

    fn build_key(key: &str) -> String {
        format!("stake.{}", key)
    }
}
