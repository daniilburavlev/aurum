use account::account::Account;
use common::bigdecimal::BigDecimal;

#[test]
fn account_debit_credit() {
    let mut account = Account::new(String::from("wallet"));

    account.debit(BigDecimal::from_str("1").unwrap()).unwrap();

    assert!(account.credit(BigDecimal::from_str("1").unwrap()).is_ok());

    match account.credit(BigDecimal::from_str("2").unwrap()) {
        Ok(_) => assert!(false, "Credit operation not validated"),
        Err(e) => assert_eq!(e, "Not enough balance"),
    }
}

#[test]
fn account_stake() {
    let mut account = Account::new(String::from("wallet"));
    account.debit(BigDecimal::from_str("2.01").unwrap()).unwrap();

    assert!(account.stake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("0.1").unwrap()).is_ok());

    match account.stake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("1").unwrap()) {
        Ok(_) => assert!(false, "Credit operation not validated"),
        Err(e) => assert_eq!(e, "Not enough balance"),
    }
}

#[test]
fn account_unstake() {
    let mut account = Account::new(String::from("wallet"));
    account.debit(BigDecimal::from_str("2.01").unwrap()).unwrap();

    assert!(account.stake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("0.1").unwrap()).is_ok());

    account.unstake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("0.1").unwrap()).unwrap();

    match account.unstake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("1").unwrap()) {
        Ok(_) => assert!(false, "Credit operation not validated"),
        Err(e) => assert_eq!(e, "Not enough stake"),
    }
}

#[test]
fn account_unstake_invalid_fee() {
    let mut account = Account::new(String::from("wallet"));
    account.debit(BigDecimal::from_str("2").unwrap()).unwrap();

    assert!(account.stake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("0.1").unwrap()).is_ok());

    match account.unstake_amount(BigDecimal::from_str("1").unwrap(), BigDecimal::from_str("1").unwrap()) {
        Ok(_) => assert!(false, "Credit operation not validated"),
        Err(e) => assert_eq!(e, "Not enough balance for fee"),
    }
}
