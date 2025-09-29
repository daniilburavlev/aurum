use libp2p::{Multiaddr, PeerId};

pub fn parse_addr(addr: &String) -> Option<(PeerId, Multiaddr)> {
    let peer_id = addr.split('/').last().unwrap_or("").to_string();
    let addr = addr.replace(&peer_id, "");
    let addr = addr[..addr.len() - 1].to_string();
    if let (Ok(peer_id), Ok(multiaddr)) = (peer_id.parse(), addr.parse()) {
        Some((peer_id, multiaddr))
    } else {
        None
    }
}
