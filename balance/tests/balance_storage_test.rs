use balance::balance::Balance;
use balance::balance_storage::BalanceStorage;
use common::bigdecimal::BigDecimal;
use std::collections::HashSet;
use tempfile::tempdir;

#[test]
fn save_get_balance() {
    let temp_dir = tempdir().unwrap();
    let db = db::open(temp_dir.path()).unwrap();
    let balance_storage = BalanceStorage::new(&db);
    let balance = Balance {
        wallet: String::from("Wallet"),
        nonce: 10,
        amount: BigDecimal::from_str("0.0001").unwrap(),
    };
    balance_storage.save(&balance).unwrap();
    let restored = balance_storage.find(balance.wallet.clone()).unwrap();
    assert_eq!(balance, restored.unwrap());

    assert!(
        balance_storage
            .find(String::from("empty"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn save_get_balances() {
    let temp_dir = tempdir().unwrap();
    let db = db::open(temp_dir.path()).unwrap();
    let balance_storage = BalanceStorage::new(&db);
    let mut wallets = HashSet::new();
    let count = 100;
    for i in 0..count {
        let balance = Balance {
            wallet: i.to_string(),
            nonce: i,
            amount: BigDecimal::from_str("0.0001").unwrap(),
        };
        wallets.insert(balance.wallet.clone());
        balance_storage.save(&balance).unwrap();
    }
    let found = balance_storage.find_all(&wallets).unwrap();
    assert_eq!(found.len(), count as usize);
}
