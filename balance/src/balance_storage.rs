use crate::balance::Balance;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

const ALL_BALANCE_WALLETS_KEY: &str = "balance.all_wallets";

pub struct BalanceStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl BalanceStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_balances_wallets()?;
        wallets.insert(balance.wallet());
        let json = serde_json::to_vec(&balance)?;
        let key = Self::build_key(&balance.wallet());
        self.save_all_balances_wallets(&wallets)?;
        self.db.put(key, &json)?;
        Ok(())
    }

    pub fn save_all(&self, balances: &Vec<Balance>) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_balances_wallets()?;
        for balance in balances {
            wallets.insert(balance.wallet.clone());
            self.save(balance)?;
        }
        self.save_all_balances_wallets(&wallets)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<HashMap<String, Balance>, Box<dyn Error>> {
        let wallets = self.all_balances_wallets()?;
        self.find_all(&wallets)
    }

    pub fn find(&self, wallet: String) -> Result<Option<Balance>, Box<dyn Error>> {
        let key = Self::build_key(&wallet);
        if let Some(json) = self.db.get(key)? {
            let value: Balance = serde_json::from_slice(&json)?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    pub fn find_all(
        &self,
        wallets: &HashSet<String>,
    ) -> Result<HashMap<String, Balance>, Box<dyn Error>> {
        let mut balances = HashMap::new();
        for wallet in wallets {
            if let Some(balance) = self.find(wallet.clone())? {
                balances.insert(wallet.clone(), balance);
            }
        }
        Ok(balances)
    }

    fn all_balances_wallets(&self) -> Result<HashSet<String>, Box<dyn Error>> {
        if let Some(wallets) = self.db.get(String::from(ALL_BALANCE_WALLETS_KEY))? {
            Ok(serde_json::from_slice(&wallets)?)
        } else {
            Ok(HashSet::new())
        }
    }

    fn save_all_balances_wallets(&self, wallets: &HashSet<String>) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(&wallets)?;
        self.db.put(String::from(ALL_BALANCE_WALLETS_KEY), json)?;
        Ok(())
    }

    fn build_key(key: &str) -> String {
        format!("balance.{}", key)
    }
}
