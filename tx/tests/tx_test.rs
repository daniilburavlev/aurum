use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[test]
fn new_tx() {
    let from = Wallet::new();
    let to = Wallet::new();
    let tx_data = TxData::new(&from, to.address_str(), String::from("0.0001"), String::from("0"), 1).unwrap();
    let tx_1 = Tx::from_tx(tx_data.clone(), String::default(), 0);
    assert!(tx_1.valid());
    let tx_2 = Tx::from_tx(tx_data, String::default(), 0);
    assert!(tx_2.valid());
    assert_eq!(tx_1.hash, tx_2.hash);
}
