use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use std::path::Path;
use std::sync::Arc;

pub fn open(filepath: &Path) -> Result<Arc<DBWithThreadMode<MultiThreaded>>, std::io::Error> {
    let mut options = Options::default();
    options.create_if_missing(true);
    let db = DBWithThreadMode::open(&options, filepath)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Arc::new(db))
}
