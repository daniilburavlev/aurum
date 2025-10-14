use balance::balance::Balance;
use common::bigdecimal::BigDecimal;
use common::biginteger::BigInt;
use operation::tx::process_tx;
use stake::stake::Stake;
use std::collections::{BTreeMap, HashMap};
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[test]
fn not_enough_balance_tx() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    let to = Wallet::new();
    let tx_data = TxData::new(&from, to.address_str(), String::from("0.001"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
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
    let tx_data = TxData::new(&from, String::from("UNSTAKE"), String::from("0.001"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
        assert_eq!(err, "Not enough balance");
    } else {
        assert!(false);
    }
}

#[test]
fn unstake() {
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
    let tx_data = TxData::new(&from, String::from("UNSTAKE"), String::from("1"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 0);
    if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
        assert!(false, "{}", err);
    }
    assert_eq!(
        stakes.get(&from.address_str()).unwrap().stake(),
        BigInt::from_str("9").unwrap()
    );
}

#[test]
fn valid_tx() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(from.address_str(), Balance {
        wallet: from.address_str(),
        amount: BigDecimal::from_str("1").unwrap(),
        nonce: 0,
    });
    let to = Wallet::new();
    let tx_data = TxData::new(&from, to.address_str(), String::from("0.001"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
        assert!(false, "{}", err);
    }
}

#[test]
fn valid_stake() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();
    let from = Wallet::new();
    balances.insert(from.address_str(), Balance {
        wallet: from.address_str(),
        amount: BigDecimal::from_str("1").unwrap(),
        nonce: 0,
    });
    let tx_data = TxData::new(&from, String::from("STAKE"), String::from("1"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    if let Some(err) = process_tx(&tx, &mut balances, &mut stakes) {
        assert!(false, "{}", err);
    }
    assert_eq!(stakes.get(&from.address_str()).unwrap().stake(), BigInt::from_str("1").unwrap());
}

#[test]
fn wrong_nonce_value() {
    let mut balances = HashMap::new();
    let mut stakes = BTreeMap::new();

    let from = Wallet::new();
    balances.insert(from.address_str(), Balance {
        wallet: from.address_str(),
        amount: BigDecimal::from_str("1").unwrap(),
        nonce: 0,
    });

    let to = Wallet::new();
    let tx_data = TxData::new(&from, to.address_str(), String::from("0.001"), 0).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 1);
    let err = process_tx(&tx, &mut balances, &mut stakes).expect("Nonce validation failure");
    assert_eq!(err, "Invalid nonce, expected: 1, was: 0");
}
