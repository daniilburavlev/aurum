use crate::behaviour::{
    BlockRequest, BlockResponse, ClientBehaviour, ClientBehaviourEvent, NonceRequest,
    NonceResponse, TxResponse,
};
use block::block::Block;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, StreamProtocol, Swarm, noise, request_response, tcp, yamux};
use tx::tx::Tx;

pub struct Client {
    swarm: Swarm<ClientBehaviour>,
    peer_id: PeerId,
}

impl Client {
    pub async fn new(node: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| {
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
                let block_behaviour =
                    request_response::json::Behaviour::<BlockRequest, BlockResponse>::new(
                        [(
                            StreamProtocol::new("/block/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                ClientBehaviour {
                    nonce: nonce_behaviour,
                    tx: tx_behaviour,
                    find_block: block_behaviour,
                }
            })?
            .build();
        let remote: Multiaddr = node.parse()?;
        swarm.dial(remote)?;
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => Ok(Self { swarm, peer_id }),
            _ => Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "").into()),
        }
    }

    pub async fn get_nonce(&mut self, address: String) -> u64 {
        self.swarm
            .behaviour_mut()
            .nonce
            .send_request(&self.peer_id, NonceRequest { address });
        match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(ClientBehaviourEvent::Nonce(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Response { response, .. } => response.nonce,
                _ => 0,
            },
            _ => 0,
        }
    }

    pub async fn find_block_by_idx(&mut self, idx: u64) -> Option<Block> {
        self.swarm
            .behaviour_mut()
            .find_block
            .send_request(&self.peer_id, BlockRequest { idx });
        match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(ClientBehaviourEvent::FindBlock(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Response { response, .. } => response.block,
                _ => None,
            },
            _ => None,
        }
    }

    pub async fn send_tx(&mut self, tx: &Tx) -> Result<(), String> {
        self.swarm
            .behaviour_mut()
            .tx
            .send_request(&self.peer_id, tx.clone());
        match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(ClientBehaviourEvent::Tx(request_response::Event::Message {
                message,
                ..
            })) => match message {
                request_response::Message::Response { response, .. } => {
                    if let Some(error) = response.error {
                        Err(error)
                    } else {
                        Ok(())
                    }
                }
                _ => Err(String::from("Internal error")),
            },
            _ => Err(String::from("Unexpected response")),
        }
    }
}
