use bigdecimal::BigDecimal;
use state::state::State;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use tx::tx::Tx;
use wallet::wallet::Wallet;

#[test]
fn test_blockchain() {
    let temp_storage = tempfile::tempdir().unwrap();
    let temp_genesis = NamedTempFile::new().unwrap();
    let wallet = wallet_with_balance(temp_genesis.path()).unwrap();
    let state = State::new(wallet.clone(), temp_storage.path()).unwrap();
    state.load_genesis(temp_genesis.path()).unwrap();

    let balance = state.balance(wallet.address()).unwrap();
    assert_eq!(balance, BigDecimal::from(500000));

    let nonce = state.nonce(wallet.address()).unwrap();
    assert_eq!(nonce, 1);

    let tx = Tx::new(&wallet, String::from("to"), String::from("100.99"), 2).unwrap();
    state.add_tx(&tx).unwrap();

    let tx = Tx::new(&wallet, String::from("to"), String::from("100.99"), 2).unwrap();
    assert!(state.add_tx(&tx).is_err());

    let genesis = state.find_block_by_idx(0).unwrap().unwrap();
    assert!(genesis.txs.is_some());
    assert_eq!(genesis.txs.unwrap().len(), 2);
}

#[test]
fn get_block_with_txs() {
    let temp_storage = tempfile::tempdir().unwrap();
    let genesis = NamedTempFile::new().unwrap();
    let wallet = wallet_with_balance(genesis.path()).unwrap();
    let state = State::new(wallet, temp_storage.path()).unwrap();
    state.load_genesis(genesis.path()).unwrap();
    let block = state.find_block_by_idx(0).unwrap().unwrap();
    assert!(block.txs.is_some());
    assert!(!block.txs.unwrap().is_empty());
}

fn wallet_with_balance(path: &Path) -> Result<Wallet, std::io::Error> {
    let wallet = Wallet::new();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let json = format!(
        "[{{\"hash\": \"GENESIS_133ec3db684243afafa83055a5f69a65\",\"from\": \"GENESIS\",\"to\": \"{}\",\"amount\": \"1000000\",\"nonce\": 1,\"timestamp\": 1009227600,\"signature\": \"GENESIS\",\"block\": 0}}, \
        {{\"hash\": \"GENESIS_52a2476f72e3491d88c7e82d5aa52469\",\"from\": \"{}\",\"to\": \"STAKE\",\"amount\": \"500000\",\"nonce\": 1,\"timestamp\": 1009227600,\"signature\": \"GENESIS\",\"block\": 0}}]",
        wallet.address(),
        wallet.address()
    );
    file.write(json.as_bytes())?;
    Ok(wallet)
}
