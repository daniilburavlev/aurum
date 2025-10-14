use balance::balance::Balance;
use block::block::Block;
use log::debug;
use operation::tx::process_tx;
use stake::stake::Stake;
use std::collections::{BTreeMap, HashMap};
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[derive(Debug)]
pub struct MemPool {
    wallet: Wallet,
    current_block: u64,
    prev_block_hash: String,
    last_event: String,
    balances: HashMap<String, Balance>,
    stakes: BTreeMap<String, Stake>,
    pending_txs: Vec<Tx>,
}

impl MemPool {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            wallet,
            current_block: 0,
            prev_block_hash: String::default(),
            last_event: String::default(),
            balances: HashMap::new(),
            stakes: BTreeMap::new(),
            pending_txs: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        prev_block_hash: String,
        current_block: u64,
        last_event: String,
        balances: HashMap<String, Balance>,
        stakes: BTreeMap<String, Stake>,
    ) {
        self.prev_block_hash = prev_block_hash;
        self.current_block = current_block;
        self.last_event = last_event;
        self.balances = balances;
        self.stakes = stakes;
    }

    pub fn add_tx(&mut self, tx_data: TxData) -> Result<Tx, String> {
        if !tx_data.valid() {
            return Err("Invalid transaction".to_string());
        }
        let tx = Tx::from_tx(tx_data, self.last_event.clone(), self.current_block);
        if let Some(err) = process_tx(&tx, &mut self.balances, &mut self.stakes) {
            Err(err)
        } else {
            self.pending_txs.push(tx.clone());
            self.last_event = tx.hash_str();
            Ok(tx)
        }
    }

    pub fn get_nonce(&mut self, wallet: String) -> u64 {
        if let Some(balance) = self.balances.get(&wallet) {
            balance.nonce
        } else {
            0
        }
    }

    pub fn new_block(&mut self, validator: String) -> Option<Block> {
        if validator != self.wallet.address_str() {
            debug!("Other validator selected");
            return None;
        }
        if let Ok(block) = Block::new(
            &self.wallet,
            self.current_block.clone(),
            self.prev_block_hash.clone(),
            self.pending_txs.clone(),
        ) {
            self.pending_txs.clear();
            Some(block)
        } else {
            None
        }
    }
}
