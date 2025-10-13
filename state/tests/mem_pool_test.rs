use balance::balance::Balance;
use common::bigdecimal::BigDecimal;
use state::state::State;
use std::collections::{BTreeMap, HashMap};
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[tokio::test]
async fn add_tx() {
    let hash = String::default();
    let current_block = 0;
    let last_event = String::default();

    let mut balances = HashMap::new();
    let stakes = BTreeMap::new();

    let wallet = Wallet::new();

    let nonce = 0;
    let amount = BigDecimal::from_str("1000").unwrap();

    balances.insert(
        hash.clone(),
        Balance {
            wallet: wallet.address_str(),
            nonce,
            amount: amount.clone(),
        },
    );

    let state = State::new(wallet.clone());
    state
        .update(hash, current_block, last_event, balances, stakes)
        .await;

    assert_eq!(nonce, state.get_nonce(wallet.address_str()).await);
    let tx = TxData::new(&wallet, String::from("to"), String::from("100"), 1).unwrap();
    let tx = state.add_tx(tx).await.unwrap();
    println!("{:?}", tx);
}
