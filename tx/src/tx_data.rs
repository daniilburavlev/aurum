use common::bigdecimal::BigDecimal;
use crypto::crypto::verify_signature;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use wallet::wallet::Wallet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxData {
    pub from: String,
    pub to: String,
    pub amount: BigDecimal,
    pub fee: BigDecimal,
    pub nonce: u64,
    pub signature: String,
}

impl TxData {
    pub fn new(
        wallet: &Wallet,
        to: String,
        amount: String,
        fee: String,
        nonce: u64,
    ) -> Result<Self, std::io::Error> {
        let amount = BigDecimal::from_str(amount.as_str())?;
        let fee = BigDecimal::from_str(fee.as_str())?;
        let mut tx = Self {
            from: wallet.address_str(),
            to,
            amount,
            fee,
            nonce,
            signature: "".to_string(),
        };
        let signature = wallet.sign(&tx.hash())?;
        tx.signature = signature;
        Ok(tx)
    }

    pub fn from(&self) -> String {
        self.from.clone()
    }

    pub fn to(&self) -> String {
        self.from.clone()
    }

    pub fn amount(&self) -> BigDecimal {
        self.amount.clone()
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn fee(&self) -> BigDecimal {
        self.fee.clone()
    }

    pub fn signature(&self) -> String {
        self.signature.clone()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.to_string().as_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hasher.finalize().into()
    }

    pub fn valid(&self) -> bool {
        match bs58::decode(&self.from).into_vec() {
            Ok(public_key) => {
                if let Ok(public_key) = public_key.try_into() {
                    verify_signature(public_key, &self.signature, &self.hash())
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    pub fn fee_amount(&self) -> BigDecimal {
        if self.fee == BigDecimal::zero() {
            return BigDecimal::zero();
        }
        self.amount.clone() / self.fee.clone()
    }
}
