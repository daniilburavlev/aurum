use crate::mem_pool::MemPool;
use balance::balance::Balance;
use block::block::Block;
use stake::stake::Stake;
use std::collections::{BTreeMap, HashMap};
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

pub struct State {
    mem_pool: futures::lock::Mutex<MemPool>,
}

impl State {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            mem_pool: futures::lock::Mutex::new(MemPool::new(wallet)),
        }
    }

    pub async fn update(
        &self,
        prev_block_hash: String,
        current_block: u64,
        last_event: String,
        balances: HashMap<String, Balance>,
        stakes: BTreeMap<String, Stake>,
    ) {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.update(prev_block_hash, current_block, last_event, balances, stakes);
    }

    pub async fn add_tx(&self, tx_data: TxData) -> Result<Tx, String> {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.add_tx(tx_data)
    }

    pub async fn get_nonce(&self, wallet: String) -> u64 {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.get_nonce(wallet)
    }

    pub async fn new_block(&self, validator: String) -> Option<Block> {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.new_block(validator)
    }
}
