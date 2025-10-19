use balance::balance::Balance;
use common::bigdecimal::BigDecimal;
use stake::stake::Stake;
use std::collections::{BTreeMap, HashMap};
use tx::tx::Tx;

const STAKE_WALLET: &str = "STAKE";
const UNSTAKE_WALLET: &str = "UNSTAKE";

pub fn process_tx(
    validator: String,
    tx: &Tx,
    balances: &mut HashMap<String, Balance>,
    stakes: &mut BTreeMap<String, Stake>,
) -> Option<String> {
    let validator = balances.entry(validator.clone()).or_insert(Balance {
        wallet: validator,
        nonce: 0,
        amount: BigDecimal::zero(),
    });
    validator.amount += tx.fee();
    if let Some(balance) = balances.get_mut(&tx.from()) {
        if tx.to() != UNSTAKE_WALLET {
            if balance.amount < tx.amount_with_fee() && tx.block != 0 {
                return Some(String::from("Not enough balance"));
            }
            balance.amount -= tx.amount_with_fee();
        } else {
            if balance.amount < tx.fee() && tx.block != 0 {
                return Some(String::from("Not enough balance"));
            }
            balance.amount += tx.amount();
            balance.amount -= tx.fee();
        }
        let expected_nonce = balance.nonce + 1;
        if tx.nonce() != expected_nonce {
            return Some(format!(
                "Invalid nonce, expected: {}, was: {}",
                expected_nonce,
                tx.nonce()
            ));
        }
        balance.nonce = tx.nonce();
    } else if tx.block != 0 {
        return Some(String::from("Not enough balance"));
    }
    if tx.to() == STAKE_WALLET {
        let stake = stakes.entry(tx.from()).or_insert(Stake::empty(tx.from()));
        if let Some(value) = tx.amount().to_bigint() {
            stake.stake += value;
        } else {
            return Some(String::from("Stake value must be bigint"));
        }
    } else if tx.to() == UNSTAKE_WALLET {
        let stake = stakes
            .entry(String::from(tx.from()))
            .or_insert(Stake::empty(tx.from()));
        if let Some(value) = tx.amount().to_bigint() {
            stake.stake -= value;
        } else {
            return Some(String::from("Stake value must be bigint"));
        }
    } else {
        let balance = balances.entry(tx.to()).or_insert(Balance::default());
        balance.wallet = tx.to();
        balance.amount += tx.amount();
    }
    None
}
