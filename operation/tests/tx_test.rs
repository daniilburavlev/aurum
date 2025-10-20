use account::account::Account;
use common::bigdecimal::BigDecimal;
use common::biginteger::BigInt;
use operation::tx::process_tx;
use std::collections::BTreeMap;
use std::process::exit;
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

const DEFAULT_VALIDATOR: &str = "";

#[test]
fn not_enough_balance_tx() {
    let mut accounts = BTreeMap::new();
    let from = Wallet::new();
    let to = Wallet::new();
    let tx_data = TxData::new(
        &from,
        to.address_str(),
        String::from("0.001"),
        String::from("1"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) {
        assert_eq!(err, "Not enough balance");
    } else {
        assert!(false);
    }
}

#[test]
fn not_enough_balance_unstake() {
    let mut accounts = BTreeMap::new();
    let from = Wallet::new();
    let tx_data = TxData::new(
        &from,
        String::from("UNSTAKE"),
        String::from("0.001"),
        String::from("1"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) {
        assert_eq!(err, "Not enough balance");
    } else {
        assert!(false);
    }
}

#[test]
fn valid_unstake() {
    let mut accounts = BTreeMap::new();
    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account
        .debit(BigDecimal::from_str("10.02").unwrap())
        .unwrap();
    account
        .stake_amount(
            BigDecimal::from_str("10").unwrap(),
            BigDecimal::from_str("0.01").unwrap(),
        )
        .unwrap();

    accounts.insert(from.address_str(), account);

    let tx_data = TxData::new(
        &from,
        String::from("UNSTAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) {
        assert!(false, "{}", err);
    }
    assert_eq!(
        accounts.get(&from.address_str()).unwrap().stake(),
        BigInt::from_str("9").unwrap()
    );
}

#[test]
fn not_enough_for_fee_unstake() {
    let mut accounts = BTreeMap::new();
    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("10").unwrap()).unwrap();

    accounts.insert(from.address_str(), account);
    let tx_data = TxData::new(
        &from,
        String::from("UNSTAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) {
        println!("{}", err);
        assert!(true);
    } else {
        assert!(false, "Balance is not validated")
    }
}

#[test]
fn not_enough_for_fee_stake() {
    let mut balances = BTreeMap::new();

    let from = Wallet::new();

    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("1").unwrap()).unwrap();

    balances.insert(from.address_str(), account);
    let tx_data = TxData::new(
        &from,
        String::from("STAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut balances) {
        println!("{}", err);
        assert!(true);
    } else {
        assert!(false, "Balance is not validated")
    }
}

#[test]
fn valid_tx() {
    let mut balances = BTreeMap::new();
    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("1").unwrap()).unwrap();

    balances.insert(from.address_str(), account);
    let to = Wallet::new();
    let tx_data = TxData::new(
        &from,
        to.address_str(),
        String::from("0.001"),
        String::from("0"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut balances) {
        assert!(false, "{}", err);
    }
}

#[test]
fn valid_stake() {
    let mut accounts = BTreeMap::new();

    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("10").unwrap()).unwrap();

    accounts.insert(from.address_str(), account);
    let tx_data = TxData::new(
        &from,
        String::from("STAKE"),
        String::from("1"),
        String::from("1"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) {
        assert!(false, "{}", err);
    }
    assert_eq!(
        accounts.get(&from.address_str()).unwrap().stake(),
        BigInt::from_str("1").unwrap()
    );
}

#[test]
fn wrong_nonce_value() {
    let mut accounts = BTreeMap::new();

    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("1").unwrap()).unwrap();

    accounts.insert(from.address_str(), account);

    let to = Wallet::new();
    let tx_data = TxData::new(
        &from,
        to.address_str(),
        String::from("0.001"),
        String::from("0"),
        0,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    let Err(err) = process_tx(DEFAULT_VALIDATOR.to_string(), &tx, &mut accounts) else {
        assert!(false, "Expect nonce validation");
        exit(-1);
    };
    assert_eq!(err, "Invalid nonce, expected: 1, was: 0");
}

#[test]
fn tx_fee() {
    let mut accounts = BTreeMap::new();

    let start_balance = String::from("0.1");
    let start_fee = String::from("0.001");

    let from = Wallet::new();
    let mut account = Account::new(from.address_str());
    account.debit(BigDecimal::from_str("100").unwrap()).unwrap();
    account.set_nonce(1).unwrap();

    let to = Wallet::new();
    accounts.insert(from.address_str(), account);
    let tx_data = TxData::new(
        &from,
        to.address_str(),
        start_balance.clone(),
        start_fee.clone(),
        2,
    )
    .unwrap();
    let validator = Wallet::new();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    process_tx(validator.address_str(), &tx, &mut accounts).unwrap();

    let validator_balance = accounts.get(&validator.address_str()).unwrap();
    let from_balance = accounts.get(&from.address_str()).unwrap();
    let to_balance = accounts.get(&to.address_str()).unwrap();

    assert_eq!(validator_balance.balance.to_plain_string(), start_fee);
    assert_eq!(from_balance.balance.to_plain_string(), "99.899");
    assert_eq!(to_balance.balance.to_plain_string(), "0.1");
}
