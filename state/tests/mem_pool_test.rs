use account::account::Account;
use common::bigdecimal::BigDecimal;
use state::state::State;
use std::collections::BTreeMap;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[tokio::test]
async fn add_tx() {
    let current_block = 0;
    let last_event = String::default();

    let mut accounts = BTreeMap::new();

    let wallet = Wallet::new();

    let nonce = 0;
    let amount = BigDecimal::from_str("1000").unwrap();

    let mut account = Account::new(wallet.address_str());
    account.debit(amount).unwrap();

    accounts.insert(account.wallet(), account.clone());

    let state = State::new(wallet.clone());
    state
        .update(account.wallet(), current_block, last_event, accounts)
        .await;

    let Some(account) = state.get_account(wallet.address_str()).await else {
        panic!("account not found");
    };
    assert_eq!(nonce, account.nonce);
    let tx = TxData::new(
        &wallet,
        String::from("to"),
        String::from("100"),
        String::from("1"),
        1,
    )
    .unwrap();
    let tx = state.add_tx(tx).await.unwrap();
    println!("{:?}", tx);
}
