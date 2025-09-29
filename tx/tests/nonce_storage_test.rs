use db::open;
use tx::nonce_storage::NonceStorage;

#[test]
fn test_nonce_save_get() -> Result<(), std::io::Error> {
    let temp_dir = tempfile::tempdir()?;
    let db = open(temp_dir.path())?;
    let nonce_storage = NonceStorage::new(&db);
    let value = 90;
    nonce_storage.save(String::from("wallet"), value)?;
    let found = nonce_storage.get(String::from("wallet"))?;
    assert_eq!(value, found);
    let zero = nonce_storage.get(String::from("wallet1"))?;
    assert_eq!(zero, 0);
    Ok(())
}
