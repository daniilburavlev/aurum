use block::block::Block;
use tx::tx::Tx;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[test]
fn test_genesis_block_creation() -> Result<(), std::io::Error> {
    let wallet = Wallet::new();
    let tx_data = TxData::new(&wallet, wallet.address_str(), String::from("1"), 1)?;
    let tx = Tx::from_tx(tx_data, String::default(), 0);
    let txs = vec![tx];

    let block1 = Block::genesis(txs.clone());
    assert_eq!(block1.idx, 0);

    let block2 = Block::genesis(txs);
    assert_eq!(block2.idx, 0);

    assert_eq!(block1.hash(), block2.hash());
    assert_eq!(block1.hash_str(), block2.hash_str());
    Ok(())
}

#[test]
fn test_new_block_creation() -> Result<(), std::io::Error> {
    let wallet = Wallet::new();
    let tx_data = TxData::new(&wallet, wallet.address_str(), String::from("1"), 1)?;
    let tx = Tx::from_tx(tx_data, String::default(), 0);
    let txs = vec![tx];

    let genesis = Block::genesis(txs.clone());
    Block::new(&wallet, 1, genesis.hash_str(), txs)?;
    Ok(())
}
