use crate::tx::Tx;
use wallet::wallet::Wallet;

#[test]
fn test_new_tx() {
    let from = Wallet::new();
    let to = Wallet::new();
    let tx = Tx::new(&from, to.address(), String::from("0.0001"), 1).unwrap();
    assert!(tx.valid());
}
