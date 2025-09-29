use block::block::Block;
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, kad, request_response};
use serde::{Deserialize, Serialize};
use tx::tx::Tx;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceRequest {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResponse {
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    pub idx: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Option<Block>,
}

#[derive(NetworkBehaviour)]
pub struct ClientBehaviour {
    pub find_block: request_response::json::Behaviour<BlockRequest, BlockResponse>,
}

#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub nonce: request_response::json::Behaviour<NonceRequest, NonceResponse>,
    pub tx: request_response::json::Behaviour<Tx, TxResponse>,
    pub find_block: request_response::json::Behaviour<BlockRequest, BlockResponse>,
}
