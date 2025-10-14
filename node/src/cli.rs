use crate::config::Config;
use crate::logger::init_logger;
use crate::node::Node;
use clap::{Parser, Subcommand};
use rpc::client::RpcClient;
use std::process::exit;
use tx::tx_data::TxData;
use wallet::wallet::Wallet;

#[derive(Parser)]
#[command(version, about, long_about = "XCHG blockchain node")]
pub struct NodeCli {
    #[command(subcommand)]
    command: NodeCmd,
}

#[derive(Subcommand)]
pub enum NodeCmd {
    #[clap(about = "Initialize chain with new genesis block")]
    Init {
        #[arg(long, value_name = "PATH", help = "Storage path")]
        storage: String,
        #[arg(long, value_name = "PATH", help = "Genesis path")]
        genesis: String,
    },
    #[clap(about = "Run node locally")]
    Run {
        #[arg(long, value_name = "PATH", help = "Config file path")]
        config: String,
    },
    #[clap(about = "Create new wallet")]
    CreateWallet {
        #[arg(
            long,
            value_name = "PATH",
            help = "Path to keystore e.g. '/user/username/.xchg'"
        )]
        keystore: String,
        #[arg(short = 's', help = "Print secret to stdout")]
        log_secret: bool,
    },
    #[clap(about = "Find block by index")]
    FindBlock {
        #[arg(long, value_name = "URL", help = "URL address of known node")]
        node: String,
        #[arg(long, value_name = "IDX", help = "Block height index")]
        idx: u64,
    },
    #[clap(about = "Find block by index")]
    NewTx {
        #[arg(long, value_name = "PATH", help = "Keystore path")]
        keystore: String,
        #[arg(long, value_name = "PATH", help = "Wallet address")]
        wallet: String,
        #[arg(long, value_name = "URL", help = "URL address of known node")]
        node: String,
        #[arg(long, value_name = "WALLET", help = "Receiver wallet address")]
        to: String,
        #[arg(long, value_name = "AMOUNT", help = "Amount of transaction")]
        amount: String,
    },
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

fn create_wallet(keystore: String, log_secret: bool) {
    let wallet = Wallet::new();
    let password = read_password();
    if let Err(_) = wallet.write(&keystore, password.as_bytes()) {
        eprintln!("Failed to write wallet!");
    } else {
        println!("Wallet created: {}", wallet.address_str());
        if log_secret {
            println!("Secret key: {}", wallet.secret_str());
        }
    }
}

fn read_password() -> String {
    println!("Enter password: ");
    rpassword::read_password().unwrap()
}

async fn find_block(node: String, idx: u64) {
    let client = RpcClient::new(node);
    if let Some(block) = client.find_block(idx).await {
        let json = serde_json::to_string_pretty(&block).unwrap();
        for line in json.lines() {
            println!("{}", line);
        }
    } else {
        eprintln!("Block not found: {}", idx);
    }
}

async fn add_tx(keystore: String, wallet: String, node: String, to: String, amount: String) {
    let password = read_password();
    let Ok(wallet) = Wallet::read(&keystore, &wallet, password.as_bytes()) else {
        eprintln!("Wallet not found");
        exit(1);
    };
    let client = RpcClient::new(node);
    let next_nonce = client.get_nonce(wallet.address_str()).await + 1;
    let Ok(tx) = TxData::new(&wallet, to, amount, next_nonce) else {
        eprintln!("Can't create new transaction");
        exit(1);
    };
    if let Some(err) = client.add_tx(tx).await {
        eprintln!("Invalid transaction: {}", err);
    } else {
        println!("Transaction successfully added");
    }
}

pub async fn start_cli() {
    let cli = NodeCli::parse();
    match cli.command {
        NodeCmd::Init { storage, genesis } => init_node(&storage, &genesis),
        NodeCmd::Run { config } => start_node(config).await,
        NodeCmd::CreateWallet {
            keystore,
            log_secret,
        } => create_wallet(keystore, log_secret),
        NodeCmd::FindBlock { node, idx } => find_block(node, idx).await,
        NodeCmd::NewTx {
            keystore,
            wallet,
            node,
            to,
            amount,
        } => add_tx(keystore, wallet, node, to, amount).await,
    }
}
