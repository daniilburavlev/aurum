use crate::config::Config;
use block::block::Block;
use libp2p::PeerId;
use log::{debug, error};
use p2p::network::Client;
use state::state::State;
use std::error::Error;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use storage::storage::Storage;
use tokio::sync::mpsc::Sender;
use tokio::task::spawn;
use tokio_cron_scheduler::{Job, JobScheduler};
use wallet::wallet::Wallet;

pub struct Node {
    http_port: i32,
    wallet: Wallet,
    address: String,
    nodes: Vec<String>,
    state: Arc<State>,
    storage: Arc<Storage>,
}

impl Node {
    pub fn init(storage_path: &str, genesis_path: &str) {
        let path = Path::new(storage_path);
        let state = Storage::new(path);
        let path = Path::new(genesis_path);
        if let Err(_) = state.load_genesis_from_file(&path) {
            eprintln!("Error opening genesis file");
            exit(1);
        }
    }

    pub async fn new(config: &Config) -> Self {
        let wallet = Self::get_wallet(config.secret());
        let storage_path = config.storage_path();
        let path = Path::new(&storage_path);
        let storage = Arc::new(Storage::new(path));
        let state = Arc::new(State::new(wallet.clone()));
        if let Some(latest_block) = storage.find_latest_block() {
            state.update(
                latest_block.hash_str(),
                latest_block.idx() + 1,
                latest_block.last_event(),
                storage.balances(),
                storage.stakes(),
            ).await;
        }
        Self {
            http_port: config.http_port(),
            address: config.address(),
            wallet,
            state,
            storage,
            nodes: config.nodes(),
        }
    }

    pub async fn start(&self) {
        let (block_tx, block_rx) = tokio::sync::mpsc::channel::<Block>(100);
        if let Err(_) = self.start_validator(block_tx).await {
            eprintln!("Error starting validator");
            exit(1)
        }
        let (mut client, event_loop) =
            p2p::network::new(self.wallet.secret(), &self.storage, &self.state, block_rx)
                .await
                .unwrap();
        spawn(event_loop.run());
        client
            .start_listening(self.address.clone().parse().unwrap())
            .await
            .expect("Failed to start listening");
        client.start_providing(self.wallet.address_str()).await;
        client.subscribe().await;

        if !self.nodes.is_empty()
            && let Some((address, peer_id)) = p2p::address::address_with_id(self.nodes[0].clone())
        {
            client.dial(peer_id.clone(), address).await.unwrap();
            self.sync_state(&mut client, peer_id).await;
        }
        rpc::server::run(
            self.http_port,
            self.wallet.address_str(),
            &self.storage,
            &self.state,
            client
        ).await;
    }

    async fn start_validator(&self, block_tx: Sender<Block>) -> Result<(), Box<dyn Error>> {
        let storage = Arc::clone(&self.storage);
        let state = Arc::clone(&self.state);
        let scheduler = JobScheduler::new().await?;
        scheduler
            .add(Job::new_async("*/12 * * * * *", move |_, _| {
                let block_tx = block_tx.clone();
                let storage = Arc::clone(&storage);
                let state = Arc::clone(&state);
                Box::pin(async move {
                    let validator = storage.current_validator().unwrap();
                    match state.new_block(validator).await {
                        Some(block) => {
                            if let Err(e) = storage.add_block(&block) {
                                error!("Error adding block: {}", e);
                            } else {
                                state.update(
                                    block.hash_str(),
                                    block.idx() + 1,
                                    block.last_event(),
                                    storage.balances(),
                                    storage.stakes(),
                                ).await;
                                if let Err(e) = block_tx.send(block).await {
                                    error!("Error sending block: {}", e);
                                }
                            }
                        }
                        None => debug!("Cannot create block!"),
                    };
                })
            })?)
            .await?;
        scheduler.start().await?;
        Ok(())
    }

    async fn sync_state(&self, client: &mut Client, peer_id: PeerId) {
        let mut synced = false;
        while !synced {
            let idx = if let Some(latest_block) = self.storage.find_latest_block() {
                latest_block.idx + 1
            } else {
                0
            };
            match client.find_block(idx, peer_id).await {
                Some(block) => {
                    if let Err(_) = self.storage.add_block(&block) {
                        break;
                    } else {
                        self.state.update(
                            block.hash_str(),
                            block.idx() + 1,
                            block.last_event(),
                            self.storage.balances(),
                            self.storage.stakes(),
                        ).await;
                    }
                }
                None => synced = true,
            }
        }
    }

    fn get_wallet(secret: String) -> Wallet {
        if let Ok(wallet) = Wallet::from_secret_str(secret) {
            return wallet;
        }
        eprintln!("Incorrect secret");
        exit(1);
    }
}
