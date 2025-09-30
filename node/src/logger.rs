use flexi_logger::FileSpec;
use std::env::home_dir;

const DEFAULT_PATH: &str = ".aurum/log";

pub fn init_logger() {
    let path = home_dir().unwrap().join(DEFAULT_PATH);
    flexi_logger::Logger::try_with_str("debug")
        .unwrap()
        .log_to_file(FileSpec::default().directory(path))
        .start()
        .unwrap();
}
