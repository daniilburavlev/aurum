use block::block::Block;
use libp2p::{Multiaddr, PeerId};
use p2p::network;
use state::state::State;
use std::sync::Arc;
use storage::storage::Storage;
use tempfile::tempdir;
use tokio::task::spawn;
use wallet::wallet::Wallet;

#[tokio::test]
pub async fn p2p_test() {
    let wallet = Wallet::new();

    let temp_storage_dir1 = tempdir().unwrap();
    let storage1 = Storage::new(temp_storage_dir1.path());
    let state1 = State::new(wallet.clone());
    let storage1 = Arc::new(storage1);
    let state1 = Arc::new(state1);

    let genesis = Block::new(&wallet, 0, String::from("0"), vec![]).unwrap();
    storage1.add_block(&genesis).unwrap();
    let (_, rx) = tokio::sync::mpsc::channel(10);

    let (mut client1, loop1) = network::new(wallet.secret(), &storage1, &state1, rx)
        .await
        .unwrap();
    spawn(loop1.run());

    let address: Multiaddr = "/ip4/127.0.0.1/tcp/18977".parse().unwrap();

    client1.start_listening(address.clone()).await.unwrap();
    client1.start_providing(wallet.address_str()).await;

    let wallet2 = Wallet::new();

    let temp_storage_dir2 = tempdir().unwrap();
    let storage2 = Storage::new(temp_storage_dir2.path());
    let state2 = State::new(wallet.clone());
    let storage2 = Arc::new(storage2);
    let state2 = Arc::new(state2);

    let (_, rx) = tokio::sync::mpsc::channel(10);
    let (mut client2, loop2) = network::new(wallet2.secret(), &storage2, &state2, rx)
        .await
        .unwrap();
    spawn(loop2.run());

    client2
        .start_listening("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .await
        .unwrap();

    let public = libp2p::identity::ecdsa::PublicKey::try_from_bytes(&wallet.address()).unwrap();
    let public = libp2p::identity::PublicKey::from(public);
    let peer_id = PeerId::from(public);
    println!("peer id: {:?}", peer_id);

    let address: Multiaddr = "/ip4/127.0.0.1/tcp/18977".parse().unwrap();
    client2
        .dial(peer_id, address.clone())
        .await
        .expect("Dial to succeed");

    let providers = client2.get_providers(wallet.address_str()).await;
    assert_eq!(providers.len(), 1);

    let peer_id = providers.iter().next().unwrap().clone();

    let nonce = client2.get_nonce(wallet.address_str(), peer_id).await;
    assert_eq!(nonce, 0);

    let block = client2.find_block(0, peer_id).await.unwrap();
    assert_eq!(block, genesis);
}
