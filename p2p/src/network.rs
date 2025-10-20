use account::account::Account;
use block::block::Block;
use common::bigdecimal::BigDecimal;
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::gossipsub::IdentTopic;
use libp2p::kad::GetProvidersOk;
use libp2p::kad::store::MemoryStore;
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm, gossipsub, kad, noise, request_response, tcp, yamux,
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use state::state::State;
use std::collections::{HashMap, HashSet, hash_map};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use storage::storage::Storage;
use tokio::sync::mpsc::Receiver;
use tx::tx::Tx;
use tx::tx_data::TxData;

#[derive(NetworkBehaviour)]
pub struct P2pBehaviour {
    kademlia: kad::Behaviour<MemoryStore>,
    gossipsub: gossipsub::Behaviour,
    get_nonce: request_response::json::Behaviour<NonceRequest, AccountResponse>,
    find_block: request_response::json::Behaviour<BlockRequest, BlockResponse>,
    add_tx: request_response::json::Behaviour<TxData, TxResponse>,
    get_fee: request_response::json::Behaviour<FeeRequest, FeeResponse>,
}

#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Subscribe,
    StartProviding {
        wallet: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        wallet: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    GetAccount {
        wallet: String,
        peer: PeerId,
        sender: oneshot::Sender<Option<Account>>,
    },
    FindBlock {
        idx: u64,
        peer: PeerId,
        sender: oneshot::Sender<Option<Block>>,
    },
    AddTx {
        data: TxData,
        peer: PeerId,
        sender: oneshot::Sender<Result<Tx, String>>,
    },
    GetFee {
        peer: PeerId,
        sender: oneshot::Sender<FeeResponse>,
    },
}

pub async fn new(
    secret: [u8; 32],
    storage: &Arc<Storage>,
    state: &Arc<State>,
    block_rx: Receiver<Block>,
) -> Result<(Client, EventLoop), Box<dyn Error>> {
    let secret = libp2p::identity::secp256k1::SecretKey::try_from_bytes(secret).unwrap();
    let keypair = libp2p::identity::secp256k1::Keypair::from(secret);
    let keypair = libp2p::identity::Keypair::from(keypair);
    let peer_id = keypair.public().to_peer_id();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let gossibsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .build()
                .map_err(tokio::io::Error::other)
                .expect("Error creating gossipsub config");
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossibsub_config,
            )
            .expect("Cannot build gossipsub");
            P2pBehaviour {
                kademlia: kad::Behaviour::new(peer_id, MemoryStore::new(key.public().to_peer_id())),
                get_nonce: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/get-nonce/0.0.1"),
                        ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
                find_block: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/find-block/0.0.1"),
                        ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
                gossipsub,
                add_tx: request_response::json::Behaviour::new(
                    [(StreamProtocol::new("/add-tx/0.0.1"), ProtocolSupport::Full)],
                    request_response::Config::default(),
                ),
                get_fee: request_response::json::Behaviour::new(
                    [(StreamProtocol::new("/get-fee/0.0.1"), ProtocolSupport::Full)],
                    request_response::Config::default(),
                ),
            }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
        },
        EventLoop::new(swarm, command_receiver, storage, state, block_rx),
    ))
}

pub struct EventLoop {
    swarm: Swarm<P2pBehaviour>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    command_receiver: mpsc::Receiver<Command>,
    pending_get_account: HashMap<OutboundRequestId, oneshot::Sender<Option<Account>>>,
    pending_add_tx: HashMap<OutboundRequestId, oneshot::Sender<Result<Tx, String>>>,
    pending_get_fee: HashMap<OutboundRequestId, oneshot::Sender<FeeResponse>>,
    pending_find_block: HashMap<OutboundRequestId, oneshot::Sender<Option<Block>>>,
    new_block_topic: IdentTopic,
    storage: Arc<Storage>,
    state: Arc<State>,
    block_rx: Receiver<Block>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<P2pBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
        storage: &Arc<Storage>,
        state: &Arc<State>,
        block_rx: Receiver<Block>,
    ) -> Self {
        Self {
            swarm,
            pending_dial: HashMap::new(),
            pending_start_providing: HashMap::new(),
            pending_get_providers: HashMap::new(),
            command_receiver,
            pending_get_account: HashMap::new(),
            pending_add_tx: HashMap::new(),
            pending_find_block: HashMap::new(),
            pending_get_fee: HashMap::new(),
            storage: Arc::clone(storage),
            state: Arc::clone(state),
            new_block_topic: IdentTopic::new("new_block"),
            block_rx,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.next() => match command {
                    Some(command) => self.handle_command(command).await,
                    None => {
                        error!("Command channel closed");
                        return
                    },
                },
                block = self.block_rx.recv() => self.handle_block(block).await,
            }
        }
    }

    async fn handle_block(&mut self, block: Option<Block>) {
        if let Some(block) = block {
            let block = serde_json::to_vec(&block).unwrap();
            if let Err(e) = self
                .swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.new_block_topic.clone(), block)
            {
                error!("Failed to publish block: {}", e);
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<P2pBehaviourEvent>) {
        match event {
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                debug!(
                    "Incoming connection '{}' from: {}, back addr: {}",
                    connection_id, local_addr, send_back_addr
                );
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!(
                    "Listening on {:?}/p2p/{}",
                    address,
                    self.swarm.local_peer_id()
                );
            }
            SwarmEvent::Behaviour(P2pBehaviourEvent::Kademlia(event)) => match event {
                kad::Event::OutboundQueryProgressed { id, result, .. } => match result {
                    kad::QueryResult::StartProviding(_) => {
                        let sender: oneshot::Sender<()> = self
                            .pending_start_providing
                            .remove(&id)
                            .expect("Completed query to be previously pending");
                        let _ = sender.send(());
                    }
                    kad::QueryResult::GetProviders(Ok(result)) => match result {
                        GetProvidersOk::FoundProviders { providers, .. } => {
                            if let Some(sender) = self.pending_get_providers.remove(&id) {
                                sender.send(providers).expect("Receiver not to be dropped");
                                self.swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .query_mut(&id)
                                    .unwrap()
                                    .finish();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            SwarmEvent::Behaviour(P2pBehaviourEvent::GetNonce(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let account = self.state.get_account(request.wallet).await;
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .get_nonce
                        .send_response(channel, AccountResponse { account })
                    {
                        error!("Failed to send nonce: {:?}", e);
                    }
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_get_account
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(response.account);
                }
            },
            SwarmEvent::Behaviour(P2pBehaviourEvent::FindBlock(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = if let Ok(block) = self.storage.find_block_by_idx(request.idx) {
                        BlockResponse { block }
                    } else {
                        BlockResponse { block: None }
                    };
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .find_block
                        .send_response(channel, response)
                    {
                        error!("Failed to send block: {:?}", e);
                    }
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_find_block
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(response.block);
                }
            },
            SwarmEvent::Behaviour(P2pBehaviourEvent::AddTx(request_response::Event::Message {
                message,
                ..
            })) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match self.state.add_tx(request).await {
                        Ok(tx) => TxResponse {
                            data: Some(tx),
                            error: None,
                        },
                        Err(err) => TxResponse {
                            data: None,
                            error: Some(err),
                        },
                    };
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .add_tx
                        .send_response(channel, response)
                    {
                        error!("Failed to send nonce: {:?}", e);
                    }
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let result = if let Some(tx) = response.data {
                        Ok(tx)
                    } else {
                        Err(response
                            .error
                            .unwrap_or(String::from("Invalid transaction")))
                    };
                    let _ = self
                        .pending_add_tx
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(result);
                }
            },
            SwarmEvent::Behaviour(P2pBehaviourEvent::GetFee(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request { channel, .. } => {
                    let fee = self.state.current_fee().await;
                    let response = FeeResponse { fee };
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .get_fee
                        .send_response(channel, response)
                    {
                        error!("Failed to send fee: {:?}", e);
                    }
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_get_fee
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(response);
                }
            },
            SwarmEvent::Behaviour(P2pBehaviourEvent::Gossipsub(event)) => match event {
                gossipsub::Event::Message { message, .. } => {
                    debug!("New block {:?}", message);
                    if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                        if let Err(e) = self.storage.add_block(&block) {
                            error!("Failed to add block: {}", e);
                        }
                    } else {
                        error!("Got invalid block!");
                    }
                }
                gossipsub::Event::Subscribed { peer_id, topic } => {
                    debug!("Peer {peer_id} subscribed to {topic}");
                }
                _ => {}
            },
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => {
                debug!("Dialing {peer_id}")
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                error!("Dialing error");
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            e => {
                debug!("Unhandled {:?}", e);
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    debug!("Dialing peer {} {}", peer_id, peer_addr);
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, peer_addr.clone());
                    self.swarm
                        .add_peer_address(peer_id.clone(), peer_addr.clone());
                    match self.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(Box::new(e)));
                        }
                    }
                }
            }
            Command::StartProviding { wallet, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(wallet.into_bytes().into())
                    .expect("No store error");
                self.pending_start_providing.insert(query_id, sender);
            }
            Command::GetProviders { wallet, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(wallet.into_bytes().into());
                self.pending_get_providers.insert(query_id, sender);
            }
            Command::GetAccount {
                wallet,
                peer,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .get_nonce
                    .send_request(&peer, NonceRequest { wallet });
                self.pending_get_account.insert(request_id, sender);
            }
            Command::FindBlock { idx, peer, sender } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .find_block
                    .send_request(&peer, BlockRequest { idx });
                self.pending_find_block.insert(request_id, sender);
            }
            Command::AddTx { data, peer, sender } => {
                let request_id = self.swarm.behaviour_mut().add_tx.send_request(&peer, data);
                self.pending_add_tx.insert(request_id, sender);
            }
            Command::Subscribe => {
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(&self.new_block_topic)
                {
                    error!("Failed to subscribe to new blocks: {:?}", e);
                }
            }
            Command::GetFee { peer, sender } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .get_fee
                    .send_request(&peer, FeeRequest {});
                self.pending_get_fee.insert(request_id, sender);
            }
        }
    }
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped");
        receiver.await.expect("Command receiver not to be dropped")
    }

    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn start_providing(&mut self, wallet: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding { wallet, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    pub async fn get_providers(&mut self, wallet: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders { wallet, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn get_account(&mut self, wallet: String, peer: PeerId) -> Option<Account> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetAccount {
                wallet,
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn add_tx(&mut self, data: TxData, peer: PeerId) -> Result<Tx, String> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::AddTx { data, peer, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn find_block(&mut self, idx: u64, peer: PeerId) -> Option<Block> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::FindBlock { idx, peer, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn subscribe(&mut self) {
        self.sender
            .send(Command::Subscribe)
            .await
            .expect("Command receiver not to be dropped.");
    }

    pub async fn get_fee(&mut self, peer: PeerId) -> FeeResponse {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetFee { peer, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonceRequest {
    pub wallet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account: Option<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRequest {
    pub idx: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Option<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxResponse {
    pub data: Option<Tx>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeResponse {
    pub fee: BigDecimal,
}
