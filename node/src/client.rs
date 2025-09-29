use crate::behaviour::{BlockRequest, BlockResponse, ClientBehaviour, ClientBehaviourEvent};
use block::block::Block;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{noise, request_response, tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm};

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
                let block_behaviour =
                    request_response::json::Behaviour::<BlockRequest, BlockResponse>::new(
                        [(
                            StreamProtocol::new("/block/0.0.1"),
                            request_response::ProtocolSupport::Full,
                        )],
                        request_response::Config::default(),
                    );
                ClientBehaviour {
                    find_block: block_behaviour,
                }
            })?
            .build();
        let remote: Multiaddr = node.parse()?;
        println!("Dialing");
        swarm.dial(remote)?;
        println!("Dialed");
        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => Ok(Self { swarm, peer_id }),
            e => {
                println!("{:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "").into())
            }
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
                e => {
                    println!("{:?}", e);
                    None
                }
            },
            e => {
                println!("{:?}", e);
                None
            }
        }
    }
}
