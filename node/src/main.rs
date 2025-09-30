use aurum::{cli, logger};

#[tokio::main]
async fn main() {
    logger::init_logger();
    cli::start_cli().await;
}
