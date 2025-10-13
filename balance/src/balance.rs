use common::bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Balance {
    pub wallet: String,
    pub nonce: u64,
    pub amount: BigDecimal,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            wallet: String::from(""),
            nonce: 0,
            amount: BigDecimal::from_str("0").unwrap(),
        }
    }
}

impl Balance {
    pub fn wallet(&self) -> String {
        self.wallet.clone()
    }
}
