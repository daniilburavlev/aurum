use crate::cli;
use clap::{Parser, Subcommand};
use p2p::client::Client;
use rpassword::read_password;
use std::env::home_dir;
use std::process::exit;
use tx::tx::Tx;
use wallet::wallet::Wallet;

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
    FindBlock {
        #[arg(long, value_name = "node")]
        node: String,
        #[arg(long, value_name = "idx")]
        idx: u64,
    },
    Restore {
        #[arg(long, value_name = "secret")]
        secret: String,
    },
}

async fn create_wallet() -> Result<(), std::io::Error> {
    let password = get_password();
    let wallet = Wallet::new();
    println!("secret: {}", wallet.hex_secret());
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
    let password = read_password()?;
    let keystore_path = home_dir().unwrap().join(DEFAULT_KEYSTORE);
    let keystore_path = keystore_path.to_str().unwrap();
    let wallet = Wallet::read(keystore_path, from.as_str(), password.as_bytes())?;
    let mut client = Client::new(address).await?;
    let nonce = client.get_nonce(from).await;
    let tx = Tx::new(&wallet, to, amount, nonce + 1)?;
    println!("Tx created: {:?}", tx);
    if let Err(e) = client.send_tx(&tx).await {
        eprintln!("Invalid transaction: {}", e);
    } else {
        println!("Transaction successfully submitted");
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

async fn find_block_by_idx(address: String, idx: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(address).await?;
    if let Some(block) = client.find_block_by_idx(idx).await {
        let json = serde_json::to_string_pretty(&block)?;
        println!("{}", json);
    } else {
        println!("No block found!");
    }
    Ok(())
}

fn restore(secret: String) {
    if let Ok(secret) = hex::decode(secret) {
        if secret.len() != 32 {
            eprintln!("Invalid secret!");
            exit(1);
        }
        let secret: [u8; 32] = secret.try_into().unwrap();
        if let Ok(wallet) = Wallet::from_secret(secret) {
            println!("Wallet: {}", wallet.address());
            let password = get_password();
            let path = home_dir().unwrap().join(DEFAULT_KEYSTORE);
            if let Err(_) = wallet.write(path.to_str().unwrap(), password.as_bytes()) {
                eprintln!("Wallet not found");
                exit(1);
            }
        }
    } else {
        eprintln!("Invalid secret!");
        exit(1);
    }
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
        WalletCmd::Restore { secret } => {
            restore(secret);
        }
    }
}

fn get_password() -> String {
    println!("Enter password:");
    if let Ok(password) = read_password() {
        password
    } else {
        eprintln!("Error reading password!");
        exit(1);
    }
}
