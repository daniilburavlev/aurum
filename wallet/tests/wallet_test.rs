use libp2p::PeerId;
use wallet::wallet::Wallet;

const WALLET_PASSWORD: &[u8] = b"password";

#[test]
fn create_write_read_wallet() {
    let temp_file = tempfile::tempdir().unwrap();
    let keystore = temp_file.path().to_str().unwrap();
    let wallet = Wallet::new();
    wallet.write(keystore, WALLET_PASSWORD).unwrap();
    let restored = Wallet::read(keystore, &wallet.address_str(), WALLET_PASSWORD).unwrap();
    assert_eq!(wallet.address(), restored.address());
}

#[test]
fn recreate_wallet() {
    let wallet = Wallet::new();
    let restored = Wallet::from_secret(wallet.secret()).unwrap();
    assert_eq!(wallet.secret(), restored.secret());
    let restored = Wallet::from_secret_str(wallet.secret_str()).unwrap();
    assert_eq!(wallet.secret(), restored.secret());
}

#[test]
fn generate_peer_id() {
    let wallet = Wallet::new();
    let public = wallet.address();
    let public = libp2p::identity::secp256k1::PublicKey::try_from_bytes(&public).unwrap();
    let public = libp2p::identity::PublicKey::from(public);
    let peer_id = PeerId::from_public_key(&public);
    println!("peer id: {:?}", peer_id);
}
