use common::bigdecimal::BigDecimal;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use storage::storage::Storage;
use tempfile::{tempdir, NamedTempFile};
use wallet::wallet::Wallet;

#[test]
fn load_genesis_and_add_valid_block() {
    let temp_dir = tempdir().unwrap();
    let storage = Storage::new(temp_dir.path());
    let temp_file = NamedTempFile::new().unwrap();
    let wallet = Wallet::new();
    wallet_with_balance(&wallet, temp_file.path()).unwrap();
    storage.load_genesis_from_file(temp_file.path()).unwrap();
    let block = storage.find_latest_block().unwrap();
    assert_eq!(block.idx, 0);
}

#[test]
fn current_validator() {
    let temp_dir = tempdir().unwrap();
    let storage = Storage::new(temp_dir.path());
    let temp_file = NamedTempFile::new().unwrap();
    let wallet = Wallet::new();
    wallet_with_balance(&wallet, temp_file.path()).unwrap();
    storage.load_genesis_from_file(temp_file.path()).unwrap();
    let validator = storage.current_validator().unwrap();
    println!("Current validator: {:?}", validator);
}

#[test]
fn regenerate_genesis() {
    let temp_dir_1 = tempdir().unwrap();
    let temp_dir_2 = tempdir().unwrap();
    let storage_1 = Storage::new(temp_dir_1.path());
    let storage_2 = Storage::new(temp_dir_2.path());
    let temp_file = NamedTempFile::new().unwrap();
    let wallet = Wallet::new();
    wallet_with_balance(&wallet, temp_file.path()).unwrap();
    storage_1.load_genesis_from_file(temp_file.path()).unwrap();
    storage_2.load_genesis_from_file(temp_file.path()).unwrap();
    let genesis_1 = storage_1.find_latest_block().unwrap();
    let genesis_2 = storage_1.find_latest_block().unwrap();
    assert_eq!(genesis_1.hash_str(), genesis_2.hash_str());
}

#[test]
fn check_genesis_state() {
    let temp_dir = tempdir().unwrap();
    let storage = Storage::new(temp_dir.path());

    let genesis_file = NamedTempFile::new().unwrap();
    let wallet = Wallet::new();
    wallet_with_balance(&wallet, genesis_file.path()).unwrap();
    storage.load_genesis_from_file(genesis_file.path()).unwrap();

    let balances = storage.accounts();
    println!("Current balances: {:?}", balances);
    assert_eq!(balances.len(), 3);

    let balance = balances.get(&wallet.address_str()).unwrap();
    assert_eq!(balance.nonce, 1);
    assert_eq!(balance.balance(), BigDecimal::from_str("500000").unwrap());
}

fn wallet_with_balance(wallet: &Wallet, path: &Path) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let json = format!(
        "[{{\"from\": \"GENESIS\",\"to\": \"{}\",\"amount\": \"1000000\",\"fee\": \"0\",\"nonce\": 1,\"signature\": \"GENESIS\"}},\
        {{\"from\": \"{}\",\"to\": \"STAKE\",\"amount\": \"500000\",\"fee\": \"0\",\"nonce\": 1,\"signature\": \"GENESIS\"}}]",
        wallet.address_str(),
        wallet.address_str()
    );
    file.write(json.as_bytes())?;
    Ok(())
}
