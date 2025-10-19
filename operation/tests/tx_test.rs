use balance::balance::Balance;
use common::bigdecimal::BigDecimal;
use common::biginteger::BigInt;
use operation::tx::process_tx;
use stake::stake::Stake;
use std::collections::{BTreeMap, HashMap};
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

const DEFAULT_VALIDATOR: &str = "";

#[test]
fn not_enough_balance_tx() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
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
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        assert_eq!(err, "Not enough balance");
    } else {
        assert!(false);
    }
}

#[test]
fn not_enough_balance_unstake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
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
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        assert_eq!(err, "Not enough balance");
    } else {
        assert!(false);
    }
}

#[test]
fn valid_unstake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            nonce: 0,
            amount: BigDecimal::from_str("0.01").unwrap(),
        },
    );
    stakes.insert(
        from.address_str(),
        Stake {
            wallet: from.address_str(),
            stake: BigInt::from_str("10").unwrap(),
        },
    );
    let tx_data = TxData::new(
        &from,
        String::from("UNSTAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        assert!(false, "{}", err);
    }
    assert_eq!(
        stakes.get(&from.address_str()).unwrap().stake(),
        BigInt::from_str("9").unwrap()
    );
}

#[test]
fn not_enough_for_fee_unstake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            nonce: 0,
            amount: BigDecimal::from_str("0").unwrap(),
        },
    );
    stakes.insert(
        from.address_str(),
        Stake {
            wallet: from.address_str(),
            stake: BigInt::from_str("10").unwrap(),
        },
    );
    let tx_data = TxData::new(
        &from,
        String::from("UNSTAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        println!("{}", err);
        assert!(true);
    } else {
        assert!(false, "Balance is not validated")
    }
}

#[test]
fn not_enough_for_fee_stake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();

    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            nonce: 0,
            amount: BigDecimal::from_str("0").unwrap(),
        },
    );
    let tx_data = TxData::new(
        &from,
        String::from("STAKE"),
        String::from("1"),
        String::from("0.01"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        println!("{}", err);
        assert!(true);
    } else {
        assert!(false, "Balance is not validated")
    }
}

#[test]
fn valid_tx() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            amount: BigDecimal::from_str("1").unwrap(),
            nonce: 0,
        },
    );
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
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        assert!(false, "{}", err);
    }
}

#[test]
fn valid_stake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            amount: BigDecimal::from_str("10").unwrap(),
            nonce: 0,
        },
    );
    let tx_data = TxData::new(
        &from,
        String::from("STAKE"),
        String::from("1"),
        String::from("1"),
        1,
    )
    .unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    ) {
        assert!(false, "{}", err);
    }
    assert_eq!(
        stakes.get(&from.address_str()).unwrap().stake(),
        BigInt::from_str("1").unwrap()
    );
}

#[test]
fn wrong_nonce_value() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();

    let from = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            amount: BigDecimal::from_str("1").unwrap(),
            nonce: 0,
        },
    );

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
    let err = process_tx(
        DEFAULT_VALIDATOR.to_string(),
        &tx,
        &mut balances,
        &mut stakes,
    )
    .expect("Nonce validation failure");
    assert_eq!(err, "Invalid nonce, expected: 1, was: 0");
}

#[test]
fn tx_fee() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();

    let start_balance = String::from("0.1");
    let start_fee = String::from("0.001");

    let from = Wallet::new();
    let to = Wallet::new();
    balances.insert(
        from.address_str(),
        Balance {
            wallet: from.address_str(),
            nonce: 1,
            amount: BigDecimal::from_str("100").unwrap(),
        },
    );
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
    process_tx(validator.address_str(), &tx, &mut balances, &mut stakes);

    let validator_balance = balances.get(&validator.address_str()).unwrap();
    let from_balance = balances.get(&from.address_str()).unwrap();
    let to_balance = balances.get(&to.address_str()).unwrap();

    assert_eq!(validator_balance.amount.to_plain_string(), start_fee);
    assert_eq!(from_balance.amount.to_plain_string(), "99.899");
    assert_eq!(to_balance.amount.to_plain_string(), "0.1");
}
