use crate::cli;
use clap::{Parser, Subcommand};
use std::env::home_dir;
use std::process::exit;
use tx::tx::Tx;
use wallet::wallet::Wallet;
use wallet_cli::client::Client;

const DEFAULT_KEYSTORE: &str = ".aurum/keystore";

#[derive(Parser)]
#[command(version, about, long_about = "xhcg-blockchain node/client")]
pub struct WalletCli {
    #[command(subcommand)]
    pub cmd: WalletCmd,
}

#[derive(Subcommand)]
pub enum WalletCmd {
    #[clap(about = "Create a new wallet")]
    Create,
    #[clap(about = "Create new transaction")]
    Tx {
        #[arg(long, value_name = "node")]
        node: String,
        #[arg(long, value_name = "from")]
        from: String,
        #[arg(long, value_name = "to")]
        to: String,
        #[arg(long, value_name = "amount")]
        amount: String,
    },
    #[clap(about = "Stake some value")]
    Stake {
        #[arg(long, value_name = "node")]
        node: String,
        #[arg(long, value_name = "from")]
        from: String,
        #[arg(long, value_name = "amount")]
        amount: String,
    },

    FindBlock{
        #[arg(long, value_name = "node")]
        node: String,
        #[arg(long, value_name = "idx")]
        idx: u64,
    },
}

async fn create_wallet() -> Result<(), std::io::Error> {
    println!("Enter password:");
    let password = rpassword::read_password()?;
    let wallet = Wallet::new();
    let keystore_path = home_dir().unwrap().join(DEFAULT_KEYSTORE);
    let keystore_path = keystore_path.to_str().unwrap();
    wallet.write(keystore_path, password.as_bytes())?;
    println!("Wallet successfully created, address: {}", wallet.address());
    Ok(())
}

async fn new_tx(
    address: String,
    from: String,
    to: String,
    amount: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter password:");
    let password = rpassword::read_password()?;
    let keystore_path = home_dir().unwrap().join(DEFAULT_KEYSTORE);
    let keystore_path = keystore_path.to_str().unwrap();
    let wallet = Wallet::read(keystore_path, from.as_str(), password.as_bytes())?;
    let mut client = Client::new(address).await?;
    let nonce = client.get_nonce(from).await;
    let tx = Tx::new(&wallet, to, amount, nonce + 1)?;
    println!("Tx created: {:?}", tx);
    if client.send_tx(&tx).await {
        println!("Transaction successfully submitted");
    } else {
        println!("Transaction invalid");
    }
    Ok(())
}

async fn stake(
    address: String,
    from: String,
    amount: String,
) -> Result<(), Box<dyn std::error::Error>> {
    new_tx(address, from, String::from("STAKE"), amount).await
}

async fn find_block_by_idx(
    address: String,
    idx: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(address).await?;
    if let Some(block) = client.find_block_by_idx(idx).await {
        let json = serde_json::to_string_pretty(&block)?;
        println!("{}", json);
    } else {
        println!("No block found!");
    }
    Ok(())
}

pub async fn start_cli() {
    let cli = cli::WalletCli::parse();
    match cli.cmd {
        WalletCmd::Create => {
            if let Err(e) = create_wallet().await {
                eprintln!("{}", e);
                exit(1);
            }
        }
        WalletCmd::Tx {
            node,
            from,
            to,
            amount,
        } => {
            if let Err(e) = new_tx(node, from, to, amount).await {
                eprintln!("{}", e);
                exit(1);
            }
        }
        WalletCmd::Stake { node, from, amount } => {
            if let Err(e) = stake(node, from, amount).await {
                eprintln!("{}", e);
                exit(1);
            }
        }
        WalletCmd::FindBlock { node, idx } => {
            if let Err(e) = find_block_by_idx(node, idx).await {
                eprintln!("{}", e);
                exit(1);
            }
        }
    }
}
