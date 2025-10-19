use db::open;
use tx::tx::Tx;
use tx::tx_data::TxData;
use tx::tx_storage::TxStorage;
use wallet::wallet::Wallet;

#[cfg(test)]
mod tx_test;

#[test]
fn test_tx_storage_save_find_by_hash() -> Result<(), std::io::Error> {
    let temp_dir = tempfile::tempdir()?;
    let db = open(temp_dir.path())?;
    let tx_storage = TxStorage::new(&db);
    let from = Wallet::new();
    let to = Wallet::new();
    let tx_data = TxData::new(&from, to.address_str(), String::from("10"), String::from("10"), 1)?;
    let tx = Tx::from_tx(tx_data, String::default(), 0);
    tx_storage.save(&vec![tx.clone()], 0)?;
    assert_eq!(tx_storage.find_latest_hash()?.unwrap(), tx.hash_str());
    Ok(())
}
