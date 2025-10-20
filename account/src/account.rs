use common::bigdecimal::BigDecimal;
use common::biginteger::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub wallet: String,
    pub balance: BigDecimal,
    pub nonce: u64,
    pub stake: BigInt,
}

impl Account {
    pub fn new(wallet: String) -> Self {
        Self {
            wallet,
            balance: BigDecimal::zero(),
            nonce: 0,
            stake: BigInt::zero(),
        }
    }

    pub fn credit(&mut self, amount: BigDecimal) -> Result<(), String> {
        if amount > self.balance {
            return Err(String::from("Not enough balance"));
        }
        self.balance -= amount;
        Ok(())
    }

    pub fn debit(&mut self, amount: BigDecimal) -> Result<(), String> {
        self.balance += amount;
        Ok(())
    }

    pub fn stake_amount(&mut self, amount: BigDecimal, fee: BigDecimal) -> Result<(), String> {
        if self.balance < (fee.clone() + amount.clone()) {
            return Err(String::from("Not enough balance"));
        }
        let Some(stake) = amount.to_bigint() else {
            return Err(String::from("Stake must be int"));
        };
        self.balance -= amount;
        self.balance -= fee;
        self.stake += stake;
        Ok(())
    }

    pub fn unstake_amount(&mut self, amount: BigDecimal, fee: BigDecimal) -> Result<(), String> {
        if self.balance < fee.clone() {
            return Err(String::from("Not enough balance for fee"));
        }
        let Some(stake) = amount.to_bigint() else {
            return Err(String::from("Stake must be int"));
        };
        if self.stake < stake.clone() {
            return Err(String::from("Not enough stake"));
        }
        self.balance += amount;
        self.balance -= fee;
        self.stake -= stake;
        Ok(())
    }

    pub fn set_nonce(&mut self, nonce: u64) -> Result<(), String> {
        if self.nonce + 1 != nonce {
            return Err(format!(
                "Invalid nonce, expected: {}, was: {}",
                self.nonce + 1,
                nonce
            ));
        }
        self.nonce = nonce;
        Ok(())
    }

    pub fn wallet(&self) -> String {
        self.wallet.clone()
    }

    pub fn balance(&self) -> BigDecimal {
        self.balance.clone()
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn stake(&self) -> BigInt {
        self.stake.clone()
    }
}
