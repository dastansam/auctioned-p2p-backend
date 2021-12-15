use ethers::prelude::LocalWallet;
use libp2p::{PeerId};
use libp2p::identity::Keypair;
use crate::Address;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeType {
    FullNode,
    LightNode,
}

/// Represents a node in the network.
pub struct P2pNode {
    pub name: String,
    pub local_key: Keypair,
    pub peer_id: PeerId,
    pub wallet: LocalWallet,
    pub peers: Vec<PeerId>,
    pub node_type: NodeType,
    pub g_port: String,
    pub eth_remote_url: String,
    pub auction_address: Address,
    pub marketplace_address: Address,
}

impl Default for P2pNode {
    fn default() -> Self {
        let name = "default-node".to_string();
        
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        let peers = vec![];

        P2pNode {
            name,
            local_key,
            peer_id,
            wallet: "0x0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
            peers,
            node_type: NodeType::LightNode,
            g_port: "50051".to_string(),
            eth_remote_url: "http://127.0.0.1:8545".to_string(),
            auction_address: Address::zero(),
            marketplace_address: Address::zero(),
        }
    }
}

impl P2pNode {
    pub fn new(
        name: String, 
        local_key: Keypair, 
        peer_id: PeerId,
        wallet: LocalWallet,
        peers: Vec<PeerId>, 
        node_type: NodeType, 
        g_port: String,
        eth_remote_url: String,
        auction_address: Address,
        marketplace_address: Address,
    ) -> Self {
        P2pNode {
            name,
            local_key,
            peer_id,
            wallet,
            peers,
            node_type,
            g_port,
            eth_remote_url,
            auction_address,
            marketplace_address,
        }
    }

    /// Set the node to full node
    pub fn set_full_node(&mut self) {
        self.node_type = NodeType::FullNode;
    }

    pub fn is_full_node(&self) -> bool {
        self.node_type == NodeType::FullNode
    }

    pub fn get_wallet (&self) -> LocalWallet {
        self.wallet.clone()
    }
}
