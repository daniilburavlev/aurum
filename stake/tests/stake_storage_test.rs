use common::biginteger::BigInt;
use stake::stake::Stake;
use stake::stake_storage::StakeStorage;
use std::env::temp_dir;

#[test]
fn save_get_stake() {
    let temp_dir = temp_dir();
    let db = db::open(temp_dir.as_path()).unwrap();
    let stake_storage = StakeStorage::new(&db);
    let stake = Stake {
        wallet: String::from("Wallet"),
        stake: BigInt::from_str("10").unwrap(),
    };
    stake_storage.save(&stake).unwrap();
    let restored = stake_storage.find(stake.wallet.clone()).unwrap();
    assert_eq!(stake, restored.unwrap());

    assert!(stake_storage.find(String::from("empty")).unwrap().is_none());
}
