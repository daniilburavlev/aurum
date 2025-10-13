use common::biginteger::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Stake {
    pub wallet: String,
    pub stake: BigInt,
}

impl Stake {
    pub fn empty(wallet: String) -> Self {
        Self {
            wallet,
            stake: BigInt::from_str("0").unwrap()
        }
    }

    pub fn wallet(&self) -> String {
        self.wallet.clone()
    }

    pub fn stake(&self) -> BigInt {
        self.stake.clone()
    }
}

impl Default for Stake {
    fn default() -> Self {
        Self {
            wallet: String::from(""),
            stake: BigInt::from_str("").unwrap(),
        }
    }
}
