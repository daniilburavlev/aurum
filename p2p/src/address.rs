use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};

pub fn address_with_id(address: String) -> Option<(Multiaddr, PeerId)> {
    if let Ok(address) = address.parse::<Multiaddr>()
        && let Some(Protocol::P2p(peer_id)) = address.iter().last()
    {
        Some((address, peer_id.clone()))
    } else {
        None
    }
}
