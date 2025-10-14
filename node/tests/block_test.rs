use state::state::State;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use storage::storage::Storage;
use tempfile::{NamedTempFile, tempdir};
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[tokio::test]
async fn add_valid_block() {
    let storage_dir = tempdir().unwrap();

    let storage = Storage::new(storage_dir.path());
    let genesis_json = NamedTempFile::new().unwrap();

    let wallet = Wallet::new();
    wallet_with_balance(&wallet, genesis_json.path()).unwrap();

    storage.load_genesis_from_file(genesis_json.path()).unwrap();

    let state = State::new(wallet.clone());
    let block = storage.find_latest_block().expect("Can't find block");
    let latest_event_hash = storage.find_latest_event_hash();
    state
        .update(
            block.hash_str(),
            1,
            latest_event_hash,
            storage.balances(),
            storage.stakes(),
        )
        .await;

    let tx1 = TxData::new(&wallet, wallet.address_str(), String::from("10"), 2).unwrap();
    let tx1 = state.add_tx(tx1).await.unwrap();
    assert!(tx1.valid());
    assert_eq!(tx1.prev_hash.clone(), block.txs.unwrap()[1].hash_str());

    let tx2 = TxData::new(&wallet, wallet.address_str(), String::from("10"), 2).unwrap();
    let tx2 = state.add_tx(tx2).await.unwrap();
    assert!(tx2.valid());
    assert_eq!(tx2.prev_hash.clone(), tx1.hash_str());

    let invalid_tx = TxData::new(&wallet, wallet.address_str(), String::from("100"), 2).unwrap();
    let err = state.add_tx(invalid_tx).await.is_err();
    assert!(err);

    let block = state.new_block(wallet.address_str()).await;
    assert!(block.clone().unwrap().valid());

    assert!(storage.add_block(&block.unwrap()).is_ok())
}

fn wallet_with_balance(wallet: &Wallet, path: &Path) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let json = format!(
        "[{{\"from\": \"GENESIS\",\"to\": \"{}\",\"amount\": \"100\",\"nonce\": 1,\"signature\": \"GENESIS\"}},\
        {{\"from\": \"{}\",\"to\": \"STAKE\",\"amount\": \"50\",\"nonce\": 1,\"signature\": \"GENESIS\"}}]",
        wallet.address_str(),
        wallet.address_str()
    );
    file.write(json.as_bytes())?;
    Ok(())
}
