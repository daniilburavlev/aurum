use account::account::Account;
use std::collections::BTreeMap;
use tx::tx::Tx;

const GENESIS_WALLET: &str = "GENESIS";
const STAKE_WALLET: &str = "STAKE";
const UNSTAKE_WALLET: &str = "UNSTAKE";

pub fn process_tx(
    validator: String,
    tx: &Tx,
    accounts: &mut BTreeMap<String, Account>,
) -> Result<(), String> {
    if tx.from() == GENESIS_WALLET && tx.block == 0 {
        let mut account = Account::new(GENESIS_WALLET.to_string());
        account.debit(tx.amount_with_fee())?;
        accounts.insert(tx.from(), account);
    }
    let Some(account) = accounts.get_mut(&tx.from()) else {
        return Err(String::from("Not enough balance"));
    };
    account.set_nonce(tx.nonce())?;
    if tx.to() == STAKE_WALLET {
        account.stake_amount(tx.amount(), tx.fee())?;
    } else if tx.to() == UNSTAKE_WALLET {
        account.unstake_amount(tx.amount(), tx.fee())?
    } else {
        account.credit(tx.amount_with_fee())?;
    }
    let account = accounts.entry(tx.to()).or_insert(Account::new(tx.to()));
    account.debit(tx.amount())?;
    let validator = accounts
        .entry(validator.clone())
        .or_insert(Account::new(validator));
    validator.debit(tx.fee())?;
    Ok(())
}
