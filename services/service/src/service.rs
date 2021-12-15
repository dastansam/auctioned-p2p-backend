use std::sync::Arc;
use std::time::Duration;
use async_std::channel::{Receiver, Sender, unbounded};
use async_std::stream;
use async_std::prelude::*;
use common_types::node::P2pNode;
use futures::{StreamExt, select};
use db::rocks::{RocksDB};
pub use db::rocks::{DB};
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::swarm::SwarmEvent;
use libp2p::{Transport};
use libp2p::core::transport::{Boxed};
use libp2p::{PeerId, Swarm, gossipsub::{IdentTopic, GossipsubEvent}, Multiaddr};
use libp2p::identity::Keypair;
use libp2p::kad::{Record, record::{Key}, Quorum};
use log::{error};
use prost::Message;

use crate::behaviour::{NodeBehaviour};
use common_types::{
    AppStorage, Gossip, NetworkMessage, OrderCommitment, 
    Storage, Error, NetworkEvent
};

pub struct P2pService {
    swarm: Swarm<NodeBehaviour>,
    db: Arc<RocksDB>,
    sender_in: Sender<NetworkMessage>,
    sender_out: Sender<NetworkEvent>,
    receiver_in: Receiver<NetworkMessage>,
    receiver_out: Receiver<NetworkEvent>,
}

impl P2pService {
    pub fn new(
        local_key: Keypair,
        db: Arc<RocksDB>,
        bootnode: Option<String>
    ) -> Self {
        // create a peer id
        let local_peer_id = PeerId::from(local_key.public());
        let transport = create_transport(local_key.clone());
        // instantiate swarm from our NodeBehaviour
        let mut swarm = Swarm::new(
            transport, 
            NodeBehaviour::new(&local_key, local_peer_id.clone(), bootnode, db.clone()), 
            local_peer_id
        );

        // listen on
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        if let Err(e) = swarm.behaviour_mut().bootstrap() {
            println!("[KAD] Couldn't bootstrap from kademlia {}", e);
        }

        // create network message senders/receivers
        let (sender_in, receiver_in) = unbounded();
        let (sender_out, receiver_out) = unbounded();
        
        P2pService {
            swarm,
            db,
            sender_in, 
            receiver_in,
            sender_out,
            receiver_out,
        }
    }

    /// Launches the p2p service
    /// Gets Node instance
    pub async fn launch(self, node: Arc<P2pNode>) {
        let mut swarm_stream = self.swarm.fuse();
        let mut network_stream = self.receiver_in.fuse();
        
        // let mut interval = async_std::stream::int
        loop {
            select! {
                swarm_event = swarm_stream.next() => match swarm_event {
                    Some(event) => match event {
                        SwarmEvent::NewListenAddr {address, .. } => {
                            println!("[SERVICE] Your node's address: {:?}/{:?}", address, swarm_stream.get_mut().local_peer_id().to_string());
                        },
                        _ => { continue; }
                    },
                    None => { break; }
                },
                network_message = network_stream.next() => match network_message {
                    Some(message) => match message {
                        NetworkMessage::GossipMessage { source, topic, message } => {
                            println!("[SERVICE] Got message! {:?}", message);
                        }
                        NetworkMessage::PingRequest { peer_id } => {
                            let gossip = swarm_stream.get_mut().behaviour_mut().gossip(IdentTopic::new("ping"), "ping ping");
                            println!("[SERVICE] Got ping request! {:?}", gossip);
                            match gossip {
                                Ok(gossip) => {
                                    println!("[SERVICE] Gossiping ping");
                                },
                                Err(e) => {
                                    println!("[SERVICE] Couldn't send gossip message {:?}", e);
                                }
                            };
                        }
                        NetworkMessage::NewSlot {address, slot} => {
                            println!("[SERVICE] New slot gossip {:?}", slot);
                            // set slot in db
                            self.db.set_slot_number(slot);
                            
                            // kademlia record
                            let record = Record {
                                key: Key::new(b"slot-number"),
                                value: slot.to_be_bytes().to_vec(),
                                publisher: None,
                                expires: None
                            };
                            // insert new slot in the distributed DHT
                            swarm_stream.get_mut()
                                .behaviour_mut()
                                .kademlia.put_record(record, Quorum::One)
                                .expect("Couldn't put record in kad");
                        }
                        NetworkMessage::NewOrderCommitment {
                            source,
                            order_commitment
                        } => {
                            println!("[SERVCE] Received new order commitment {:?}", order_commitment);
                            // encode order commitment
                            let mut buff = Vec::new();
                            buff.reserve(order_commitment.encoded_len());
                            order_commitment.encode(&mut buff).unwrap();

                            println!("[SERVICE] Gossiping order commitment: {:?}", buff.clone());

                            let gossip_order = swarm_stream
                                .get_mut()
                                .behaviour_mut()
                                .gossip(
                                    IdentTopic::new("order_commitment"),
                                    buff
                                );
                            // attempt to gossip order commitment
                            match gossip_order {
                                Ok(gossip) => {
                                    println!("[SERVICE] Gossiping order commitment:");
                                },
                                Err(e) => {
                                    println!("[SERVICE] Couldn't send gossip message {:?}", e);
                                }
                            };

                            if self.db.put_order_commitment(order_commitment).is_err() {
                                error!("Couldn't store order commitment in db");
                            };
                        },
                        NetworkMessage::RemoveOrder {
                            id
                        } => {
                            println!("[SERVICE] Removing order {:?}", id);
                            let gossip_order = swarm_stream
                                .get_mut()
                                .behaviour_mut()
                                .gossip(
                                    IdentTopic::new("cancel_order"),
                                    id.as_bytes()
                                );
                            // attempt to gossip order commitment
                            match gossip_order {
                                Ok(gossip) => {
                                    println!("[SERVICE] Gossiping cancel order commitment:");
                                },
                                Err(e) => {
                                    println!("[SERVICE] Couldn't send gossip message {:?}", e);
                                }
                            };

                            // if self.db.delete_order_commitment(&id).is_err() {
                            //     error!("Couldn't delete order commitment in db");
                            // };
                        },
                        NetworkMessage::CurrentProcessor {address} => {
                            println!("Current processor {:?}", address);
                            // self.db.set_processor_address(address);
                        }
                        _ => println!("Unhandled request"),
                    }
                    None => { break; }
                },
            };
        }
    }

    pub fn network_receiver(&self) -> Receiver<NetworkEvent> {
        self.receiver_out.clone()
    }

    pub fn network_sender(&self) -> Sender<NetworkMessage> {
        self.sender_in.clone()
    }


}

/// emit event to the network
pub async fn emit_event(sender: &Sender<NetworkEvent>, event: NetworkEvent) {
    if sender.send(event).await.is_err() {
        error!("Couldn't send event to node behaviour");
    }
}

/// Create a new transport for the p2p service communication
pub fn create_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let transport = {
        let tcp = libp2p::tcp::TcpConfig::new().nodelay(true);
        let ws = libp2p::websocket::WsConfig::new(tcp.clone()).or_transport(tcp);
        async_std::task::block_on(
            libp2p::dns::DnsConfig::system(ws)).unwrap()
    };

    // generate dh keys
    let keys = libp2p::noise::Keypair::<libp2p::noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Failed generating noise key");
    
    // prepare authentication configuration
    let auth_config = libp2p::noise::NoiseConfig::xx(keys).into_authenticated();

    // prepare mplex and yamux configs
    let mplex_config = libp2p::mplex::MplexConfig::new();
    let yamux_config = libp2p::yamux::YamuxConfig::default();

    // prepare select upgrade order
    let final_config = libp2p::core::upgrade::SelectUpgrade::new(yamux_config, mplex_config);

    transport
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(auth_config)
        .multiplex(final_config)
        .timeout(Duration::from_secs(33))
        .boxed()
}