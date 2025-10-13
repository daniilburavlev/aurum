use crate::config::Config;
use flexi_logger::FileSpec;

pub fn init_logger(config: &Config) {
    let logs_config = config.logs();
    flexi_logger::Logger::try_with_str(logs_config.level())
        .unwrap()
        .log_to_file(FileSpec::default().directory(logs_config.dir()))
        .log_to_stderr()
        .start()
        .unwrap();
}
