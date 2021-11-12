use std::time::Duration;
use async_std::channel::{Receiver, Sender, unbounded};
use futures::{StreamExt, select};
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::swarm::SwarmEvent;
use libp2p::{Transport};
use libp2p::core::transport::{Boxed};
use libp2p::{PeerId, Swarm, gossipsub::IdentTopic};
use libp2p::identity::Keypair;
use crate::behaviour::{NodeBehaviour};

// pub enum NetworkEvent {
//     GossipMessage {
//         source: PeerId,
//         message: SignedMessage
//     }
// }

// pub enum RPCAPIMethods {
//     List()
// }


#[derive()]


#[derive(Debug)]
pub enum NetworkMessage {
    GossipMessage {
        topic: IdentTopic,
        message: Vec<u8>
    },
    PingRequest {
        peer_id: PeerId,
    },
    // RPCAPIRequest {
    //     method: RPCAPIMethods
    // }
}

pub struct P2pService {
    swarm: Swarm<NodeBehaviour>,
    sender_in: Sender<NetworkMessage>,
    sender_out: Sender<NetworkMessage>,
    receiver_in: Receiver<NetworkMessage>,
    receiver_out: Receiver<NetworkMessage>,
}

impl P2pService {
    pub fn new(
        local_key: Keypair,
    ) -> Self {
        // create a peer id
        let local_peer_id = PeerId::from(local_key.public());
        let transport = create_transport(local_key.clone());

        // instantiate swarm from our NodeBehaviour
        let mut swarm = Swarm::new(
            transport, 
            NodeBehaviour::new(&local_key, local_peer_id.clone()), 
            local_peer_id
        );

        // listen on
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        if let Err(e) = swarm.behaviour_mut().bootstrap() {
            warn!("Couldn't bootstrap from kademlia {}", e);
        }

        // create network message senders/receivers
        let (sender_in, receiver_in) = unbounded();
        let (sender_out, receiver_out) = unbounded();
        
        P2pService {
            swarm,
            sender_in, 
            receiver_in,
            sender_out,
            receiver_out,
        }
    }

    /// Launches the p2p service
    pub async fn launch(self) {
        let mut swarm_stream = self.swarm.fuse();
        let mut network_stream = self.receiver_in.fuse();

        // let mut interval = async_std::stream::int
        loop {
            select! {
                swarm_event = swarm_stream.next() => match swarm_event {
                    Some(event) => match event {
                        SwarmEvent::NewListenAddr {address, .. } => {
                            println!("Listened new addr {:?}", address);
                        },
                        _ => { continue; }
                    }
                    None => { break; }
                },
                network_message = network_stream.next() => match network_message {
                    Some(message) => match message {
                        NetworkMessage::GossipMessage { topic, message } => {
                            println!("Got message! {:?}", message);
                        }
                        NetworkMessage::PingRequest { peer_id } => {
                            println!("Got ping request from {:?}", peer_id);
                        }
                        _ => println!("Unhandled request"),
                    }
                    None => { break; }
                }
            };
        }
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