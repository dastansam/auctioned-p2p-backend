mod utils;
mod cli;

use async_std::task;
use cli::{Cli, Config};
use db::rocks::RocksDB;
use grpc::start_g_rpc;
use libp2p::identity;
use structopt::StructOpt;
use common_types::{PeerId, node::{P2pNode, NodeType}, LocalWallet, Signer};
use std::sync::Arc;
use p2p_service::P2pService;
use web3::{Web3};
use crate::cli::get_home_dir;


#[async_std::main]
async fn main() {
    let Cli {options} = Cli::from_args();
    
    match options.to_config() {
        // if user has supplied arguments, we launch the node from configuration
        Ok(config) => {
            println!("Launching node with config: {:?}", config);
            let p2p_node = from_config(config);
            run(p2p_node).await;
        },
        // otherwise we launch node with default configuration
        Err(e) => {
            println!("Launching node with default configuration...");
            let p2p_node = P2pNode::default();
            run(p2p_node).await;
        }
    }
}

/// Starts the p2p node
pub async fn run(node: P2pNode) {
    let db_dir = format!(
        "{}/{}", 
        get_home_dir() + "/.nft-node", 
        format!("db/{}", &node.name),
    );

    let db = RocksDB::open(&db_dir)
        .expect("Failed to open db");
    
    let db = Arc::new(db);

    let service = P2pService::new(node.local_key.clone(), db.clone(), None);

    let network_receiver = service.network_receiver();
    let network_sender = service.network_sender();

    let g_rpc_endpoint = format!("127.0.0.1:{}", &node.g_port);
    let peer_id = node.peer_id.clone();

    // Chain related values
    let eth_remote_url = node.eth_remote_url.clone();
    let auction = node.auction_address.clone();
    let marketplace = node.marketplace_address.clone();

    let node_wallet = node.get_wallet();
    // public address
    let local_wallet_public = node_wallet.address();

    let node = Arc::new(node);
    let p2p = task::spawn(async {
        service.launch(node).await;
    });

    // web3 service
    let web3_service = Web3::new(
        eth_remote_url,
        network_sender.clone(),
        node_wallet,
        auction,
        marketplace,
        None,
        db.clone(),
        g_rpc_endpoint.clone(),
    ).await;

    let g_rpc = task::spawn(async move {
        // start the p2p service
        println!("[GRPC] Starting gRPC service at {}", g_rpc_endpoint);
        start_g_rpc(
            peer_id, 
            Arc::clone(&db), 
            network_sender.clone(), 
            &g_rpc_endpoint, 
            local_wallet_public
        ).await
    });

    //spawns the web3 subscription service
    let web3_task = task::spawn(async move {
        println!("[WEB3] Starting web3 subscription service");
        web3_service.launch_subscriptions().await
    });

    utils::block_until_sigint().await;

    p2p.cancel().await;
    g_rpc.cancel().await;
    web3_task.cancel().await;
}


/// New node from configuration
/// By default every node is a full node
pub fn from_config(config: Config) -> P2pNode {
    // generate local keys and peer id
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from_public_key(local_key.public());

    println!("[NODE] Local peer id: {:?}", local_peer_id);

    // create a wallet from the mnemonic
    let wallet: LocalWallet = match config.private_key.parse::<LocalWallet>() {
        Ok(wallet) => wallet,
        // essentially this should never happen
        Err(e) => panic!("Failed to instantiate wallet: {}", e),
    };

    P2pNode {
        name: config.node_id,
        local_key,
        peer_id: local_peer_id,
        wallet,
        peers: vec![],
        node_type: NodeType::LightNode,
        g_port: config.g_rpc_port,
        eth_remote_url: config.eth_remote_url,
        auction_address: config.auction_address,
        marketplace_address: config.marketplace_address,
    }
}