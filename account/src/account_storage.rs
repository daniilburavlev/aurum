use crate::account::Account;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::sync::Arc;

const ALL_ACCOUNT_WALLET_KEY: &str = "account.all_wallets";

pub struct AccountStorage {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl AccountStorage {
    pub fn new(db: &Arc<DBWithThreadMode<MultiThreaded>>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub fn save(&self, account: &Account) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_balances_wallets()?;
        wallets.insert(account.wallet());
        let json = serde_json::to_vec(&account)?;
        let key = Self::build_key(&account.wallet());
        self.save_all_balances_wallets(&wallets)?;
        self.db.put(key, &json)?;
        Ok(())
    }

    pub fn save_all(&self, balances: &Vec<Account>) -> Result<(), Box<dyn Error>> {
        let mut wallets = self.all_balances_wallets()?;
        for balance in balances {
            wallets.insert(balance.wallet.clone());
            self.save(balance)?;
        }
        self.save_all_balances_wallets(&wallets)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<BTreeMap<String, Account>, Box<dyn Error>> {
        let wallets = self.all_balances_wallets()?;
        self.find_all(&wallets)
    }

    pub fn find(&self, wallet: String) -> Result<Option<Account>, Box<dyn Error>> {
        let key = Self::build_key(&wallet);
        if let Some(json) = self.db.get(key)? {
            let value: Account = serde_json::from_slice(&json)?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    pub fn find_all(
        &self,
        wallets: &HashSet<String>,
    ) -> Result<BTreeMap<String, Account>, Box<dyn Error>> {
        let mut balances = BTreeMap::new();
        for wallet in wallets {
            if let Some(balance) = self.find(wallet.clone())? {
                balances.insert(wallet.clone(), balance);
            }
        }
        Ok(balances)
    }

    fn all_balances_wallets(&self) -> Result<HashSet<String>, Box<dyn Error>> {
        if let Some(wallets) = self.db.get(String::from(ALL_ACCOUNT_WALLET_KEY))? {
            Ok(serde_json::from_slice(&wallets)?)
        } else {
            Ok(HashSet::new())
        }
    }

    fn save_all_balances_wallets(&self, wallets: &HashSet<String>) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(&wallets)?;
        self.db.put(String::from(ALL_ACCOUNT_WALLET_KEY), json)?;
        Ok(())
    }

    fn build_key(key: &str) -> String {
        format!("balance.{}", key)
    }
}
