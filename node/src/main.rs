
use async_std::{io, task};
use futures::prelude::*;
use libp2p::{PeerId, identity};
use std::{
    error::Error,
    task::{Context, Poll},
};
use node::service::P2pService;

#[async_std::main]
async fn main() {
    // generate local keys and peer id
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from_public_key(local_key.public());

    let service = P2pService::new(local_key);

    println!("Local peer id: {:?}", local_peer_id);

    service.launch().await;
}