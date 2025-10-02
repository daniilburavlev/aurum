use crate::address::parse_addr;
use crate::config::Config;
use crate::p2p::P2pServer;
use block::block::Block;
use log::{debug, error};
use p2p::client::Client;
use state::state::State;
use std::env::home_dir;
use std::error::Error;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_cron_scheduler::{Job, JobScheduler};
use wallet::wallet::Wallet;

pub struct Node {
    wallet: Wallet,
    port: i32,
    nodes: Vec<String>,
    state: Arc<State>,
}

impl Node {
    pub fn init(storage_path: &str, genesis_path: &str) {
        let path = Path::new(storage_path);
        if let Ok(state) = State::new(Wallet::new(), path) {
            let path = Path::new(genesis_path);
            if let Err(_) = state.load_genesis(&path) {
                eprintln!("Error opening genesis file");
                exit(1);
            }
        } else {
            eprintln!("Error initializing node state");
            exit(1)
        }
    }

    pub async fn new(config: &Config) -> Self {
        let wallet = Self::get_wallet(config.secret());
        if let Some(mut path) = home_dir() {
            path.push(config.storage_path());
            let state = match State::new(wallet.clone(), &path) {
                Ok(state) => Arc::new(state),
                Err(_) => {
                    eprintln!("Error initializing node state");
                    exit(1);
                }
            };
            let port = config.port();
            let node = Self {
                port,
                wallet,
                state,
                nodes: config.nodes(),
            };
            if !config.nodes().is_empty() {
                node.sync_state(config.nodes().get(0).unwrap().to_owned())
                    .await;
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

    fn get_wallet(secret: String) -> Wallet {
        if let Ok(decoded) = hex::decode(secret) {
            if let Ok(secret) = decoded.try_into() {
                if let Ok(wallet) = Wallet::from_secret(secret) {
                    return wallet;
                }
            }
        }
        eprintln!("Incorrect secret");
        exit(1);
    }
}
