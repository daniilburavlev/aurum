use account::account::Account;
use account::account_storage::AccountStorage;
use block::block::Block;
use block::block_storage::BlockStorage;
use common::biginteger::BigInt;
use log::{debug, error};
use operation::tx::process_tx;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::process::exit;
use tx::tx::Tx;
use tx::tx_data::TxData;
use tx::tx_storage::TxStorage;

const FIRST_EVENT_HASH: [u8; 32] = [0u8; 32];

pub struct Storage {
    tx_storage: TxStorage,
    account_storage: AccountStorage,
    block_storage: BlockStorage,
}

impl Storage {
    pub fn new(path: &Path) -> Self {
        match db::open(path) {
            Ok(db) => Self {
                tx_storage: TxStorage::new(&db),
                block_storage: BlockStorage::new(&db),
                account_storage: AccountStorage::new(&db),
            },
            Err(e) => {
                eprintln!("Cannot initialize local storage: {}", e);
                exit(1);
            }
        }
    }

    pub fn load_genesis_from_file(&self, genesis_path: &Path) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.block_storage.find_by_idx(0)? {
            return Ok(());
        }
        let json = fs::read_to_string(genesis_path)?;
        let txs_data: Vec<TxData> = serde_json::from_str(&json)?;
        self.load_genesis(txs_data)
    }

    pub fn load_genesis(&self, txs_data: Vec<TxData>) -> Result<(), Box<dyn Error>> {
        let txs = Self::build_genesis_txs(txs_data)?;
        let mut accounts: BTreeMap<String, Account> = BTreeMap::new();

        for tx in &txs {
            process_tx("GENESIS".to_string(), tx, &mut accounts)?;
        }
        let accounts: Vec<Account> = accounts.into_values().collect();
        self.account_storage.save_all(&accounts)?;
        self.tx_storage.save(&txs, 0)?;
        let genesis = Block::genesis(txs);
        self.block_storage.save(&genesis)?;
        Ok(())
    }

    fn build_genesis_txs(txs_data: Vec<TxData>) -> Result<Vec<Tx>, Box<dyn Error>> {
        let mut txs = Vec::new();
        let mut latest_hash = bs58::encode(FIRST_EVENT_HASH).into_string();
        for tx_data in txs_data {
            let tx = Tx::from_tx(tx_data, latest_hash.clone(), 0);
            latest_hash = tx.hash_str();
            txs.push(tx);
        }
        Ok(txs)
    }

    pub fn add_block(&self, block: &Block) -> Result<(), Box<dyn Error>> {
        debug!("Adding block: {:?}", block);
        if let Some(latest) = self.block_storage.find_latest()? {
            let expected_idx = latest.idx + 1;
            if expected_idx != block.idx {
                return Err(format!(
                    "Invalid block index, expected: {}, was: {}",
                    expected_idx, block.idx
                )
                .into());
            }
        }
        if let Some(txs) = block.txs() {
            if let Some(latest_hash) = self.tx_storage.find_latest_hash()?
                && let Some(first) = txs.first()
                && latest_hash != first.prev_hash()
            {
                return Err(format!(
                    "PoH error, expected: {}, was: {}",
                    latest_hash,
                    first.prev_hash()
                )
                .into());
            }
            if self.save_txs(&txs, block.validator(), block.idx)? {
                self.block_storage.save(&block)?;
            } else {
                return Err("Invalid transactions".into());
            }
        }
        if let Err(e) = self.block_storage.save(&block) {
            return Err(format!("Cannot add block to storage: {}", e).into());
        }
        Ok(())
    }

    pub fn accounts(&self) -> BTreeMap<String, Account> {
        if let Ok(stakes) = self.account_storage.load_all() {
            stakes
        } else {
            BTreeMap::new()
        }
    }

    pub fn find_latest_event_hash(&self) -> String {
        if let Ok(Some(hash)) = self.tx_storage.find_latest_hash() {
            hash
        } else {
            Self::default_empty_hash()
        }
    }

    fn default_empty_hash() -> String {
        let data = [0u8; 32];
        bs58::encode(data).into_string()
    }

    fn save_txs(
        &self,
        txs: &Vec<Tx>,
        validator: String,
        block_idx: u64,
    ) -> Result<bool, Box<dyn Error>> {
        let mut wallets = HashSet::new();
        for tx in txs {
            if !tx.valid() {
                return Ok(false);
            }
            wallets.insert(tx.from());
            wallets.insert(tx.to());
        }
        let mut accounts = self.account_storage.find_all(&wallets)?;
        for tx in txs {
            if let Err(err) = process_tx(validator.clone(), &tx, &mut accounts) {
                debug!("Invalid tx: {}", err);
                return Ok(false);
            }
        }
        let accounts: Vec<Account> = accounts.into_values().collect();
        self.account_storage.save_all(&accounts)?;
        self.tx_storage.save(txs, block_idx)?;
        Ok(true)
    }

    pub fn find_block_by_idx(&self, idx: u64) -> Result<Option<Block>, Box<dyn Error>> {
        if let Some(mut block) = self.block_storage.find_by_idx(idx)? {
            let txs = self.tx_storage.find_by_block_idx(block.idx())?;
            block.txs = Some(txs);
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    pub fn find_latest_block(&self) -> Option<Block> {
        if let Ok(Some(mut block)) = self.block_storage.find_latest() {
            block.txs = Some(self.tx_storage.find_by_block_idx(block.idx).unwrap());
            Some(block)
        } else {
            error!("Cannot find latest block");
            None
        }
    }

    pub fn current_validator(&self) -> Result<String, Box<dyn Error>> {
        if let Some(block) = self.block_storage.find_latest()? {
            let hash = block.hash();
            let hash = Self::hash_to_int(hash);
            let stakes = self.account_storage.load_all()?;
            let index = BigInt::from_u64(hash).unwrap() % Self::total_stake(&stakes);
            let mut latest = BigInt::zero();
            for (_, stake) in stakes {
                if stake.stake() + latest.clone() > index {
                    return Ok(stake.wallet());
                }
                latest = latest + stake.stake();
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, "No latest block").into())
    }

    pub fn find_wallet_txs(&self, wallet: String) -> Vec<Tx> {
        if let Ok(txs) = self.tx_storage.find_wallet_txs(wallet) {
            txs
        } else {
            Vec::new()
        }
    }

    fn total_stake(stakes: &BTreeMap<String, Account>) -> BigInt {
        let mut total = BigInt::from_str("0").unwrap();
        for (_, stake) in stakes {
            total += stake.stake()
        }
        total
    }

    fn hash_to_int(data: [u8; 32]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}
