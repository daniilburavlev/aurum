use wallet::wallet::Wallet;

const WALLET_PASSWORD: &[u8] = b"password";

#[test]
fn create_write_read_wallet() {
    let temp_file = tempfile::tempdir().unwrap();
    let keystore = temp_file.path().to_str().unwrap();
    let wallet = Wallet::new();
    wallet.write(keystore, WALLET_PASSWORD).unwrap();
    let restored = Wallet::read(keystore, &wallet.address(), WALLET_PASSWORD).unwrap();
    assert_eq!(wallet.address(), restored.address());
}
