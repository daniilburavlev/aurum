use crate::mem_pool::MemPool;
use account::account::Account;
use block::block::Block;
use common::bigdecimal::BigDecimal;
use std::collections::BTreeMap;
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
        accounts: BTreeMap<String, Account>,
    ) {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.update(prev_block_hash, current_block, last_event, accounts);
    }

    pub async fn add_tx(&self, tx_data: TxData) -> Result<Tx, String> {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.add_tx(tx_data)
    }

    pub async fn get_account(&self, wallet: String) -> Option<Account> {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.get_account(wallet)
    }

    pub async fn new_block(&self, validator: String) -> Option<Block> {
        let mut mem_pool = self.mem_pool.lock().await;
        mem_pool.new_block(validator)
    }
    
    pub async fn current_fee(&self) -> BigDecimal {
        let mem_pool = self.mem_pool.lock().await;
        mem_pool.current_fee()
    }
}
