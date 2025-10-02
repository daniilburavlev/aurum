use crate::config::Config;
use crate::node::Node;
use clap::{Parser, Subcommand};
use wallet::wallet::Wallet;
use crate::logger::init_logger;

#[derive(Parser)]
#[command(version, about, long_about = "Aurum blockchain node")]
pub struct NodeCli {
    #[command(subcommand)]
    command: NodeCmd,
}

#[derive(Subcommand)]
pub enum NodeCmd {
    Init {
        #[arg(long, value_name = "storage")]
        storage: String,
        #[arg(long, value_name = "genesis")]
        genesis: String,
    },
    Run {
        #[arg(long, value_name = "config")]
        config: String,
    },
    Secret,
}

async fn start_node(path: String) {
    let config = Config::read(&path);
    init_logger(&config);
    let node = Node::new(&config).await;
    node.start().await;
}

fn init_node(storage_path: &str, genesis_path: &str) {
    Node::init(storage_path, genesis_path);
}

fn new_secret() {
    let wallet = Wallet::new();
    let secret = wallet.hex_secret();
    println!("Generated secret: {}", secret);
}

pub async fn start_cli() {
    let cli = NodeCli::parse();
    match cli.command {
        NodeCmd::Init { storage, genesis } => init_node(&storage, &genesis),
        NodeCmd::Run { config } => start_node(config).await,
        NodeCmd::Secret => new_secret(),
    }
}
