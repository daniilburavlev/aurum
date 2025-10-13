use block::block::Block;
use block::block_storage::BlockStorage;
use tempfile::tempdir;
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[test]
fn test_block_save() {
    let temp_dir = tempdir().unwrap();
    let wallet = Wallet::new();
    let tx_data = TxData::new(&wallet, wallet.address_str(), String::from("0.001"), 1).unwrap();
    let tx = Tx::from_tx(tx_data, String::default(), 0);
    let txs = vec![tx];
    let block = Block::genesis(txs);

    let db = db::open(temp_dir.path()).unwrap();
    let block_storage = BlockStorage::new(&db);
    block_storage.save(&block).unwrap();

    if let Some(found) = block_storage.find_by_idx(0).unwrap() {
        assert_eq!(found.hash_str(), block.hash_str());
    } else {
        assert!(false, "Not found");
    }

    if let Some(found) = block_storage.find_by_hash(block.hash_str()).unwrap() {
        assert_eq!(found.hash_str(), block.hash_str());
    } else {
        assert!(false, "Not found");
    }

    if let Ok(Some(latest)) = block_storage.find_latest() {
        assert_eq!(block.hash_str(), latest.hash_str());
    } else {
        assert!(false, "Not found");
    }
}
