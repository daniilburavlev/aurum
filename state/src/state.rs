use crate::stake::Stake;
use bigdecimal::num_bigint::{BigInt, ToBigInt};
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use block::block::Block;
use block::block_storage::BlockStorage;
use db::open;
use log::{debug, error};
use sha2::Digest;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use tx::nonce_storage::NonceStorage;
use tx::tx::Tx;
use tx::tx_storage::TxStorage;
use wallet::wallet::Wallet;

const STAKE_WALLET: &str = "STAKE";
const UNSTAKE_WALLET: &str = "UNSTAKE";

pub struct State {
    wallet: Wallet,
    tx_storage: TxStorage,
    nonce_storage: NonceStorage,
    block_storage: BlockStorage,
}

impl State {
    pub fn new(
        wallet: Wallet,
        storage_path: &Path,
    ) -> Result<Self, std::io::Error> {
        let db = open(storage_path)?;
        let state = Self {
            wallet,
            tx_storage: TxStorage::new(&db),
            nonce_storage: NonceStorage::new(&db),
            block_storage: BlockStorage::new(&db),
        };
        Ok(state)
    }

    pub fn load_genesis(&self, genesis_path: &Path) -> Result<(), std::io::Error> {
        if let None = self.block_storage.find_by_idx(0)? {
            let json = fs::read_to_string(genesis_path)?;
            let txs: BTreeSet<Tx> = serde_json::from_str(&json)?;
            let genesis = Block::genesis(txs.clone());
            for tx in txs {
                self.tx_storage.save(&tx)?;
                self.nonce_storage.save(tx.from(), tx.nonce())?
            }
            self.block_storage.save(&genesis)?;
        }
        Ok(())
    }

    pub fn find_latest(&self) -> Result<Option<Block>, std::io::Error> {
        self.block_storage.find_latest()
    }

    pub fn nonce(&self, wallet: String) -> Result<u64, std::io::Error> {
        self.nonce_storage.get(wallet)
    }

    pub fn balance(&self, wallet: String) -> Result<BigDecimal, std::io::Error> {
        let txs = self.tx_storage.find_wallet_txs(wallet.clone())?;
        let mut balance = BigDecimal::zero();
        for tx in txs {
            if tx.to() == wallet || tx.to() == UNSTAKE_WALLET {
                balance += tx.amount();
            } else {
                balance -= tx.amount();
            }
        }
        Ok(balance)
    }

    pub fn add_tx(&self, tx: &Tx) -> Result<(), std::io::Error> {
        let nonce = self.nonce(tx.from())?;
        if nonce + 1 != tx.nonce() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid nonce value, expected: {}", nonce + 1),
            ));
        }
        if !tx.valid() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid transaction signature",
            ));
        }
        if tx.to() == UNSTAKE_WALLET {
            return if let Some(amount) = tx.amount().to_bigint() {
                if let Some(stake) = self.wallet_stake(tx.from())
                    && stake.stake() >= amount
                {
                    self.tx_storage.save(tx)?;
                    self.nonce_storage.save(tx.from(), tx.nonce())?;
                    return Ok(());
                }
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Not enough stake",
                ))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "The value must be an integer",
                ))
            };
        } else {
            let balance = self.balance(tx.from())?;
            if balance < tx.amount() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Not enough balance, current: {}", balance),
                ));
            }
            self.tx_storage.save(tx)?;
            self.nonce_storage.save(tx.from(), tx.nonce())?;
        }
        Ok(())
    }

    pub fn add_block(&self, block: &Block) -> Result<(), std::io::Error> {
        if block.idx != 0 && !block.valid() {
            error!("Invalid block: {:?}", block);
            return Ok(())
        }
        debug!("{:?}", block);
        if let Some(txs) = block.txs() {
            for tx in txs {
                self.nonce_storage.save(tx.from(), tx.nonce())?;
                self.tx_storage.save(&tx)?;
            }
        }
        self.block_storage.save(block)?;
        Ok(())
    }

    pub fn find_block_by_idx(&self, idx: u64) -> Result<Option<Block>, std::io::Error> {
        if let Some(mut block) = self.block_storage.find_by_idx(idx)? {
            let txs = self.tx_storage.find_by_block_idx(idx)?;
            block.txs = Some(txs);
            return Ok(Some(block));
        }
        Ok(None)
    }

    pub fn wallet_stake(&self, wallet: String) -> Option<Stake> {
        if let Ok(txs) = self.tx_storage.find_wallet_txs(wallet.clone()) {
            let mut stake = BigInt::zero();
            for tx in txs {
                if tx.to() == STAKE_WALLET {
                    stake += tx.amount().to_bigint().unwrap();
                } else if tx.to() == UNSTAKE_WALLET {
                    stake -= tx.amount().to_bigint().unwrap();
                }
            }
            return Stake::new(wallet, stake);
        }
        None
    }

    pub fn stakes(&self) -> Result<Vec<Stake>, std::io::Error> {
        let stakes = self
            .tx_storage
            .find_wallet_txs(String::from(STAKE_WALLET))?;
        let unstakes = self
            .tx_storage
            .find_wallet_txs(String::from(UNSTAKE_WALLET))?;
        let mut stakes_by_wallet: HashMap<String, BigInt> = HashMap::new();
        for tx in stakes {
            let stake_by_wallet = stakes_by_wallet.entry(tx.from()).or_insert(BigInt::zero());
            *stake_by_wallet += tx.amount().to_bigint().unwrap();
        }
        for tx in unstakes {
            let stake_by_wallet = stakes_by_wallet.entry(tx.from()).or_insert(BigInt::zero());
            *stake_by_wallet -= tx.amount().to_bigint().unwrap();
        }
        let mut result = Vec::new();
        let zero = BigInt::zero();
        for (wallet, stake) in stakes_by_wallet {
            if stake != zero {
                if let Some(stake) = Stake::new(wallet, stake) {
                    result.push(stake);
                }
            }
        }
        Ok(result)
    }

    pub fn proof_of_stake(&self) -> Result<Block, std::io::Error> {
        let latest_block = self.block_storage.find_latest()?.unwrap();
        let stakes = self.stakes()?;
        let total_stake = Self::total_stake(&stakes);
        let validator = self.select_validator(latest_block.hash_str(), &stakes, &total_stake);
        if validator == self.wallet.address() {
            self.create_block()
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Other validator selected",
            ))
        }
    }

    fn select_validator(
        &self,
        block_hash: String,
        stakes: &Vec<Stake>,
        total_stake: &BigInt,
    ) -> String {
        let stakes_hashes: Vec<[u8; 32]> =
            stakes.clone().iter().map(|staker| staker.hash()).collect();
        let merkle_tree =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&stakes_hashes);
        let merkle_root = merkle_tree.root().unwrap();

        let mut hasher = sha2::Sha256::new();
        hasher.update(hex::decode(block_hash).unwrap().as_slice());
        hasher.update(merkle_root);
        let hash: [u8; 32] = hasher.finalize().into();
        let hash = Self::hash_to_int(hash);
        let index = BigInt::from_u64(hash).unwrap() % total_stake;

        let mut latest = BigInt::zero();
        for stake in stakes {
            if stake.stake() + latest.clone() > index {
                return stake.wallet();
            }
            latest = latest + stake.stake();
        }
        String::from("")
    }

    pub fn create_block(&self) -> Result<Block, std::io::Error> {
        let latest_block = self.block_storage.find_latest()?.unwrap();
        let pending_txs = self.tx_storage.find_pending()?;
        let block = Block::new(
            &self.wallet,
            latest_block.idx + 1,
            latest_block.hash_str(),
            pending_txs.clone(),
        )?;
        self.tx_storage.update_pending(&pending_txs, block.idx)?;
        self.block_storage.save(&block)?;
        Ok(block)
    }

    fn total_stake(stakes: &Vec<Stake>) -> BigInt {
        let mut total_stake = BigInt::zero();
        for stake in stakes {
            total_stake += stake.stake();
        }
        total_stake
    }

    fn hash_to_int(data: [u8; 32]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}
