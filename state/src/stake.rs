use bigdecimal::num_bigint::BigInt;
use sha2::Digest;

const MINIMUM_STAKE: u64 = 10;

#[derive(Debug, Clone)]
pub struct Stake {
    wallet: String,
    stake: BigInt,
}

impl Stake {
    pub fn new(wallet: String, stake: BigInt) -> Option<Self> {
        if stake < BigInt::from(MINIMUM_STAKE) {
            return None;
        }
        Some(Self { wallet, stake })
    }

    pub fn wallet(&self) -> String {
        self.wallet.clone()
    }

    pub fn stake(&self) -> BigInt {
        self.stake.clone()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.wallet.as_bytes());
        hasher.update(self.stake.to_string().as_bytes());
        hasher.finalize().into()
    }
}
