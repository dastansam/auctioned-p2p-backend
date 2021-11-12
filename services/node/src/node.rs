// //! A basic p2p node that spawns GossipSub, mdns and Kademlia protocols
// //!
// //!
// //! TO-DO
// //!
// //!

// use async_std::{io, task};
// use futures::{FutureExt, TryFutureExt, prelude::*};
// use libp2p::core::muxing::StreamMuxerBox;
// use libp2p::core::transport::Boxed;
// use libp2p::kad::record::store::MemoryStore;
// use libp2p::kad::{
//     record::Key, AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
//     Quorum, Record,
// };
// use libp2p::gossipsub::{self, Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic, MessageAuthenticity, MessageId};

// use libp2p::swarm::{ExpandedSwarm, NetworkBehaviour};
// use libp2p::{
//     development_transport, identity,
//     mdns::{Mdns, MdnsConfig, MdnsEvent},
//     swarm::{NetworkBehaviourEventProcess, SwarmEvent},
//     NetworkBehaviour, PeerId, Swarm,
// };

// use std::collections::hash_map::DefaultHasher;
// use std::{
//     error::Error,
//     task::{Context, Poll},
// };
// use std::hash::{Hash, Hasher};

// use std::time::Duration;



// pub fn launch_node(
//     transport: Boxed<(PeerId, StreamMuxerBox)>,
//     explicit_id: Option<&PeerId>
// ) -> Result<(), Box<dyn Error>>{
//     let topic = IdentTopic::new("p2p-node");
//     // Create a swarm to manage peers and events.
//     let mut swarm = {
//         // Create a random key for ourselves.
//         let local_key = identity::Keypair::generate_ed25519();
//         let local_peer_id = PeerId::from(local_key.public());

//         // create message id function for gossipsub
//         let message_id_fn = |message: &GossipsubMessage| {
//             let mut hasher = DefaultHasher::new();
//             message.data.hash(&mut hasher);
//             MessageId::from(hasher.finish().to_string())
//         };

//         // Gossipsub configuration
//         let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
//             .heartbeat_interval(Duration::from_secs(15))
//             .validation_mode(gossipsub::ValidationMode::Strict)
//             .message_id_fn(message_id_fn)
//             .build() 
//             .expect("Valid configuration");
        
//         let gossipsub: Gossipsub = 
//             Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
//                 .expect("correct config");
//         // add static id 
//         // gossipsub.add_explicit_peer(explicit_id.unwrap_or());

//         // Create a Kademlia behaviour.
//         let store = MemoryStore::new(local_peer_id);
//         let kademlia = Kademlia::new(local_peer_id, store);
//         let mdns = task::block_on(Mdns::new(MdnsConfig::default()))?;
        
//         let mut behaviour = NodeBehaviour { kademlia, mdns, gsub: gossipsub };

//         // subscribe to node topic
//         behaviour.gsub.subscribe(&topic).unwrap();

//         Swarm::new(transport, behaviour, local_peer_id)
//     };
    
//     swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
//     // task::spawn(async {
//     //     rocket::build().mount("/", routes![ping, pinger]).launch().await
//     // });

//     // Kick it off.
//     task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
//         loop {
//             match swarm.poll_next_unpin(cx) {
//                 Poll::Ready(Some(event)) => {
//                     match event {
//                         SwarmEvent::NewListenAddr { address, .. } => {
//                             println!("Listening on {:?}", address);
//                         },
//                         _ => println!("TO-DO event handler")
//                     }
//                 }
//                 Poll::Ready(None) => return Poll::Ready(Ok(())),
//                 Poll::Pending => break,
//             }
//         }
//         Poll::Pending
//     }))
// }

// // async fn full_node() -> Result<(), Box<dyn Error>> {
// //     env_logger::init();
// //     // Read full lines from stdin
// //     let mut stdin = io::BufReader::new(io::stdin()).lines();

// //     // Listen on all interfaces and whatever port the OS assigns.
// // }

