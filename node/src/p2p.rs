use crate::address::parse_addr;
use crate::behaviour::{
    BlockRequest, BlockResponse, NodeBehaviour, NodeBehaviourEvent, NonceRequest, NonceResponse,
    TxResponse,
};
use block::block::Block;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::identity::Keypair;
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    gossipsub, identity, kad, noise, request_response, tcp, yamux, PeerId, StreamProtocol, Swarm,
};
use log::{debug, error};
use state::state::State;
use std::error::Error;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tx::tx::Tx;
use wallet::wallet::Wallet;

pub struct P2pServer {
    port: i32,
    state: Arc<State>,
    swarm: Swarm<NodeBehaviour>,
    block_rx: Receiver<Block>,
    tx_topic: IdentTopic,
    block_topic: IdentTopic,
}

impl P2pServer {
    pub fn new(
        wallet: &Wallet,
        state: &Arc<State>,
        port: i32,
        nodes: Option<Vec<String>>,
        block_rx: Receiver<Block>,
    ) -> Self {
        Self {
            port,
            state: Arc::clone(state),
            swarm: Self::build_swarm(wallet, &nodes).unwrap(),
            block_rx,
            tx_topic: IdentTopic::new("tx"),
            block_topic: IdentTopic::new("block"),
        }
    }

    fn build_swarm(
        validator: &Wallet,
        nodes: &Option<Vec<String>>,
    ) -> Result<Swarm<NodeBehaviour>, Box<dyn Error>> {
        let secret = identity::ecdsa::SecretKey::try_from_bytes(validator.secret())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let keypair = identity::ecdsa::Keypair::from(secret);
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
        let swarm = libp2p::SwarmBuilder::with_existing_identity(Keypair::from(keypair))
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                let gossibsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(tokio::io::Error::other)?;
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossibsub_config,
                )?;
                let peer_id = PeerId::from(key.public());
                println!("PeerID: {:?}", peer_id);
                let store = MemoryStore::new(peer_id);
                let kademlia_config = kad::Config::default();
                let mut kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);
                if let Some(nodes) = nodes {
                    for addr in nodes {
                        if let Some((peer_id, addr)) = parse_addr(addr) {
                            kademlia.add_address(&peer_id, addr);
                            if let Err(e) = kademlia.bootstrap() {
                                eprintln!("Failed to bootstrap Kademlia");
                                error!("{:?}", e);
                                exit(1);
                            }
                        }
                    }
                }
                let nonce_behaviour =
                    request_response::json::Behaviour::<NonceRequest, NonceResponse>::new(
                        [(
                            StreamProtocol::new("/nonce/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                let tx_behaviour = request_response::json::Behaviour::<Tx, TxResponse>::new(
                    [(
                        StreamProtocol::new("/tx/0.0.1"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );
                let find_block_behaviour =
                    request_response::json::Behaviour::<BlockRequest, BlockResponse>::new(
                        [(
                            StreamProtocol::new("/block/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                Ok(NodeBehaviour {
                    gossipsub,
                    kademlia,
                    nonce: nonce_behaviour,
                    tx: tx_behaviour,
                    find_block: find_block_behaviour,
                })
            })?
            .build();
        Ok(swarm)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.block_topic)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.tx_topic)?;

        self.swarm
            .listen_on(format!("/ip4/0.0.0.0/tcp/{}", self.port).parse()?)?;
        self.swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        loop {
            select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                },
                event = self.block_rx.recv() => {
                    self.publish_block(event).await;
                }
            }
        }
    }

    async fn publish_block(&mut self, block: Option<Block>) {
        if let Some(block) = block {
            if let Ok(json) = serde_json::to_string(&block) {
                debug!("{}", serde_json::to_string_pretty(&json).unwrap());
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(self.block_topic.clone(), json)
                {
                    error!("Failed to publish block: {:?}", e);
                }
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<NodeBehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: _,
                message_id: _,
                message,
            })) => {
                self.process_topic_message(&message).await;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Nonce(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request_id: _,
                    request,
                    channel,
                } => {
                    let nonce = self.state.nonce(request.address.clone());
                    self.swarm
                        .behaviour_mut()
                        .nonce
                        .send_response(
                            channel,
                            NonceResponse {
                                nonce: nonce.unwrap(),
                            },
                        )
                        .unwrap();
                }
                request_response::Message::Response {
                    request_id: _,
                    response: _,
                } => {}
            },
            SwarmEvent::Behaviour(NodeBehaviourEvent::Tx(request_response::Event::Message {
                message,
                ..
            })) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match self.state.add_tx(&request) {
                        Ok(_) => {
                            let json = serde_json::to_string(&request).unwrap();
                            if let Err(e) = self
                                .swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(self.tx_topic.clone(), json)
                            {
                                error!("Error publishing to swarm: {:?}", e);
                            }
                            TxResponse { error: None }
                        }
                        Err(e) => TxResponse {
                            error: Some(format!("{:?}", e)),
                        },
                    };
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .tx
                        .send_response(channel, response)
                    {
                        error!("Error sending response: {:?}", e);
                    }
                }
                _ => {}
            },
            SwarmEvent::Behaviour(NodeBehaviourEvent::FindBlock(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    if let Ok(block) = self.state.find_block_by_idx(request.idx) {
                        let response = BlockResponse { block };
                        if let Err(e) = self
                            .swarm
                            .behaviour_mut()
                            .find_block
                            .send_response(channel, response)
                        {
                            error!("Error sending response: {:?}", e);
                        }
                    }
                }
                _ => {}
            },
            swarm_event => debug!("{:?}", swarm_event),
        }
    }

    async fn process_topic_message(&self, message: &gossipsub::Message) {
        let topic = message.topic.clone();
        if topic == self.tx_topic.hash() {
            let tx: Tx =
                serde_json::from_str(String::from_utf8(message.clone().data).unwrap().as_str())
                    .unwrap();
            if let Err(e) = self.state.add_tx(&tx) {
                error!("Error sending message: {:?}", e);
            }
        } else if topic == self.block_topic.hash() {
            let block: Block =
                serde_json::from_str(String::from_utf8(message.clone().data).unwrap().as_str())
                    .unwrap();
            match self.state.add_block(&block) {
                Err(e) => error!("Error adding block: {:?}", e),
                _ => {}
            }
        }
    }
}
