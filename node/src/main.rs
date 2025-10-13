use xchg::cli;

#[tokio::main]
async fn main() {
    cli::start_cli().await;
}
