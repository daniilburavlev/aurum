use balance::balance::Balance;
use balance::balance_storage::BalanceStorage;
use block::block::Block;
use block::block_storage::BlockStorage;
use common::biginteger::BigInt;
use log::{debug, error};
use operation::tx::process_tx;
use stake::stake::Stake;
use stake::stake_storage::StakeStorage;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::process::exit;
use tx::tx::Tx;
use tx::tx_data::TxData;
use tx::tx_storage::TxStorage;

const FIRST_EVENT_HASH: [u8; 32] = [0u8; 32];
const GENESIS_WALLET: &str = "GENESIS";
const STAKE_WALLET: &str = "STAKE";

pub struct Storage {
    tx_storage: TxStorage,
    balance_storage: BalanceStorage,
    block_storage: BlockStorage,
    stake_storage: StakeStorage,
}

impl Storage {
    pub fn new(path: &Path) -> Self {
        match db::open(path) {
            Ok(db) => Self {
                tx_storage: TxStorage::new(&db),
                block_storage: BlockStorage::new(&db),
                balance_storage: BalanceStorage::new(&db),
                stake_storage: StakeStorage::new(&db),
            },
            Err(e) => {
                eprintln!("Cannot initialize local storage: {}", e);
                exit(1);
            }
        }
    }

    pub fn load_genesis(&self, genesis_path: &Path) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.block_storage.find_by_idx(0)? {
            return Ok(());
        }
        let json = fs::read_to_string(genesis_path)?;
        let txs = Self::build_genesis_txs(json)?;
        let mut balances: HashMap<String, Balance> = HashMap::new();
        let mut stakes: BTreeMap<String, Stake> = BTreeMap::new();
        for tx in &txs {
            if tx.from() == GENESIS_WALLET {
                let balance = balances.entry(tx.to()).or_insert(Default::default());
                balance.wallet = tx.to();
                balance.amount += tx.amount();
            } else if tx.to() == STAKE_WALLET {
                let balance = balances.entry(tx.from()).or_insert(Default::default());
                balance.wallet = tx.from();
                balance.amount -= tx.amount();
                let stake = stakes.entry(tx.from()).or_insert(Stake::empty(tx.from()));
                stake.stake += tx.amount().to_bigint().unwrap();
            }
        }
        let balances: Vec<Balance> = balances.into_values().collect();
        let stakes: Vec<Stake> = stakes.into_values().collect();
        self.balance_storage.save_all(&balances)?;
        self.stake_storage.save_all(&stakes)?;
        self.tx_storage.save(&txs, 0)?;
        let genesis = Block::genesis(txs);
        self.block_storage.save(&genesis)?;
        Ok(())
    }

    fn build_genesis_txs(json: String) -> Result<Vec<Tx>, Box<dyn Error>> {
        let txs_data: Vec<TxData> = serde_json::from_str(&json)?;
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
            if latest.idx + 1 != block.idx {
                return Ok(());
            }
        }
        if let Some(txs) = block.txs() {
            if let Some(latest_hash) = self.tx_storage.find_latest_hash()?
                && let Some(first) = txs.first()
                && latest_hash != first.prev_hash()
            {
                return Ok(());
            }
            if self.save_txs(&txs, block.idx)? {
                self.block_storage.save(&block)?;
            }
        }
        if let Err(e) = self.block_storage.save(&block) {
            error!("Cannot add block to storage: {}", e);
        }
        Ok(())
    }

    pub fn balances(&self) -> HashMap<String, Balance> {
        if let Ok(stakes) = self.balance_storage.load_all() {
            stakes
        } else {
            HashMap::new()
        }
    }

    pub fn stakes(&self) -> BTreeMap<String, Stake> {
        if let Ok(stakes) = self.stake_storage.load_all() {
            stakes
        } else {
            BTreeMap::new()
        }
    }

    pub fn find_latest_event_hash(&self) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(hash) = self.tx_storage.find_latest_hash()? {
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }

    fn save_txs(&self, txs: &Vec<Tx>, block: u64) -> Result<bool, Box<dyn Error>> {
        let mut wallets = HashSet::new();
        for tx in txs {
            if !tx.valid() {
                return Ok(false);
            }
            wallets.insert(tx.from());
            wallets.insert(tx.to());
        }
        let mut balances = self.balance_storage.find_all(&wallets)?;
        let mut stakes = self.stake_storage.find_all(&wallets)?;
        for tx in txs {
            if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
                debug!("Invalid tx: {}", err);
                return Ok(false);
            }
        }
        let balances: Vec<Balance> = balances.into_values().collect();
        let stakes: Vec<Stake> = stakes.into_values().collect();
        self.balance_storage.save_all(&balances)?;
        self.stake_storage.save_all(&stakes)?;
        self.tx_storage.save(txs, block)?;
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
            let stakes = self.stake_storage.load_all()?;
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

    fn total_stake(stakes: &BTreeMap<String, Stake>) -> BigInt {
        let mut total = BigInt::from_str("0").unwrap();
        for (_, stake) in stakes {
            total += stake.stake.clone()
        }
        total
    }

    fn hash_to_int(data: [u8; 32]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}
