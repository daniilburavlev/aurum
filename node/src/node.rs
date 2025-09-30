use crate::address::parse_addr;
use crate::client::Client;
use crate::p2p::P2pServer;
use block::block::Block;
use state::state::State;
use std::env::home_dir;
use std::error::Error;
use std::io;
use std::io::Read;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use log::{debug, error};
use tokio::sync::mpsc::Sender;
use tokio_cron_scheduler::{Job, JobScheduler};
use wallet::wallet::Wallet;

const DEFAULT_PORT: i32 = 0;
const DEFAULT_KEYSTORE: &str = ".aurum/keystore";
const DEFAULT_STORAGE_PATH: &str = ".aurum/storage";

pub struct Node {
    wallet: Wallet,
    port: i32,
    nodes: Option<Vec<String>>,
    state: Arc<State>,
}

impl Node {
    pub fn init(genesis_path: &str) {
        if let Some(mut path) = home_dir() {
            path.push(DEFAULT_STORAGE_PATH);
            if let Ok(state) = State::new(Wallet::new(), &path) {
                let path = Path::new(genesis_path);
                if let Err(_) = state.load_genesis(&path) {
                    eprintln!("Error opening genesis file");
                    exit(1);
                }
            } else {
                eprintln!("Error initializing node state");
                exit(1)
            }
        } else {
            eprintln!("Error getting home dir, check access");
            exit(1)
        }
    }

    pub async fn new(
        port: Option<i32>,
        wallet: Option<String>,
        nodes: Option<Vec<String>>,
    ) -> Self {
        let wallet = Self::get_wallet(wallet);
        if let Some(mut path) = home_dir() {
            path.push(DEFAULT_STORAGE_PATH);
            let state = match State::new(wallet.clone(), &path) {
                Ok(state) => Arc::new(state),
                Err(_) => {
                    eprintln!("Error initializing node state");
                    exit(1);
                }
            };
            let port = port.unwrap_or(DEFAULT_PORT);
            let node = Self {
                port,
                wallet,
                state,
                nodes: nodes.clone(),
            };
            if let Some(nodes) = nodes
                && !nodes.is_empty()
            {
                node.sync_state(nodes.get(0).unwrap().to_owned()).await;
            }
            return node;
        }
        eprintln!("Error initializing node state");
        exit(1);
    }

    pub async fn start(&self) {
        let (block_tx, block_rx) = tokio::sync::mpsc::channel::<Block>(100);
        if let Err(_) = self.start_validator(block_tx).await {
            eprintln!("Error starting validator");
            exit(1)
        }
        if let Err(e) = P2pServer::new(
            &self.wallet,
            &self.state,
            self.port,
            self.nodes.clone(),
            block_rx,
        )
        .start()
        .await
        {
            eprintln!("Error starting P2p: {}", e);
            exit(1);
        }
    }

    async fn start_validator(&self, block_tx: Sender<Block>) -> Result<(), Box<dyn Error>> {
        let state = Arc::clone(&self.state);
        let scheduler = JobScheduler::new().await?;
        scheduler
            .add(Job::new_async("*/12 * * * * *", move |_, _| {
                let block_tx = block_tx.clone();
                let state = Arc::clone(&state);
                Box::pin(async move {
                    match state.proof_of_stake() {
                        Ok(block) => match block_tx.send(block).await {
                            Err(e) => error!("Error sending block: {:?}", e),
                            _ => {}
                        },
                        Err(e) => debug!("Cannot create block: {}", e),
                    };
                })
            })?)
            .await?;
        scheduler.start().await?;
        Ok(())
    }

    async fn sync_state(&self, node: String) {
        let mut synced = false;
        if let Some((_, addr)) = parse_addr(&node) {
            if let Ok(mut client) = Client::new(addr.to_string()).await {
                while !synced {
                    if let Ok(latest_block) = self.state.find_latest() {
                        let idx = if let Some(latest_block) = latest_block {
                            latest_block.idx + 1
                        } else {
                            0
                        };
                        match client.find_block_by_idx(idx).await {
                            Some(block) => {
                                println!("block: {:?}", block);
                                if let Err(_) = self.state.add_block(&block) {
                                    break;
                                }
                            }
                            None => synced = true,
                        }
                    }
                }
            }
        }
    }

    fn get_wallet(wallet: Option<String>) -> Wallet {
        if let Some(mut path) = home_dir() {
            path.push(DEFAULT_KEYSTORE);
            let path = path.to_str().unwrap();
            if let Some(wallet) = wallet {
                match Wallet::read(path, &wallet, Self::read_password().as_ref()) {
                    Ok(wallet) => wallet,
                    Err(_) => {
                        eprintln!("Error reading wallet from path");
                        exit(1);
                    }
                }
            } else {
                println!("Wallet is not specified, create new one? [y/n]: ");
                if !Self::get_option() {
                    exit(0);
                }
                let password = Self::read_password();
                let wallet = Wallet::new();
                if let Err(_) = wallet.write(path, password.as_bytes()) {
                    eprintln!("Error creating wallet");
                    exit(1);
                }
                println!("Wallet successfully created: {}", wallet.address());
                wallet
            }
        } else {
            eprintln!("Error getting home dir, check access");
            exit(1);
        }
    }

    fn read_password() -> String {
        println!("Enter password");
        if let Ok(password) = rpassword::read_password() {
            password
        } else {
            eprintln!("Error reading password");
            exit(1);
        }
    }

    fn get_option() -> bool {
        let stdin = io::stdin();
        let mut bytes = stdin.bytes();
        if let Some(Ok(byte)) = bytes.next() {
            let ch = byte as char;
            if ch == 'y' {
                return true;
            } else if ch == 'n' {
                return false;
            }
        }
        eprintln!("Error reading from stdin");
        exit(1);
    }
}
