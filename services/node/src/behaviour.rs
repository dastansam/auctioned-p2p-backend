use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use async_std::task;
use libp2p::PeerId;
use libp2p::gossipsub::error::{PublishError, SubscriptionError};
use libp2p::identity::Keypair;
use libp2p::kad::{AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryId, QueryResult, Record,};
use libp2p::gossipsub::{self, Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic, MessageAuthenticity, MessageId, Topic};

use libp2p::mdns::MdnsConfig;
use libp2p::swarm::{NetworkBehaviour};
use libp2p::{
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmEvent},
    NetworkBehaviour
};
use libp2p::kad::record::store::MemoryStore;


// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub(crate) struct NodeBehaviour {
    kademlia: Kademlia<MemoryStore>,
    mdns: Mdns,
    gsub: Gossipsub
}

impl NetworkBehaviourEventProcess<MdnsEvent> for NodeBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(list) = event {
            for (peer_id, multiaddr) in list {
                self.kademlia.add_address(&peer_id, multiaddr);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for NodeBehaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message{ 
                propagation_source,
                message_id,
                message 
            } => {
                    println!(
                        "Received a message: {} from peer:{:?}",
                        String::from_utf8_lossy(&message.data),
                        propagation_source
                    );
                    // do_sth_with_new_order(message, propagation_source)
            },
            GossipsubEvent::Subscribed{
                peer_id,
                topic
            } => {
                println!("connected to peer");
            }
            GossipsubEvent::Unsubscribed{
                peer_id,
                topic
            } => {
                println!("disconnected from peer");
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for NodeBehaviour {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::OutboundQueryCompleted { result, .. } => match result {
                QueryResult::GetProviders(Ok(ok)) => {
                    for peer in ok.providers {
                        println!(
                            "Peer {:?} provides key {:?}",
                            peer,
                            std::str::from_utf8(ok.key.as_ref()).unwrap()
                        );
                    }
                }
                QueryResult::GetProviders(Err(err)) => {
                    eprintln!("Failed to get providers: {:?}", err);
                }
                QueryResult::GetRecord(Ok(ok)) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        println!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {:?}", err);
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    println!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {:?}", err);
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    println!(
                        "Successfully put provider record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::StartProviding(Err(err)) => {
                    eprintln!("Failed to put provider record: {:?}", err);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl NodeBehaviour {
    pub fn new(local_key: &Keypair, peer_id: PeerId) -> Self {
        // create topic for subscription
        let topic = IdentTopic::new("p2p-node");

        // create message id function for gossipsub
        let message_id_fn = |message: &GossipsubMessage| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        // Gossipsub configuration
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(15))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build() 
            .expect("Valid configuration");
        
        let gossipsub: Gossipsub = 
            Gossipsub::new(MessageAuthenticity::Signed(local_key.clone()), gossipsub_config)
                .expect("incorrect config");
        
        // add static id 
        // gossipsub.add_explicit_peer(explicit_id.unwrap_or());

        // Create a Kademlia behaviour.
        let store = MemoryStore::new(peer_id);
        let kademlia = Kademlia::new(peer_id, store);
        let mdns = task::block_on(Mdns::new(MdnsConfig::default())).unwrap();
        
        let mut behaviour = NodeBehaviour { kademlia, mdns, gsub: gossipsub };

        // subscribe to node topic
        behaviour.gsub.subscribe(&topic).unwrap();

        behaviour
    }

    /// Bootstrap kademlia
    pub fn bootstrap(&mut self) -> Result<QueryId, String> {
        self.kademlia.bootstrap().map_err(|e| e.to_string())
    }

    /// Gossip message across the network
    pub fn gossip(&mut self, topic: IdentTopic, message: impl Into<Vec<u8>>) -> Result<MessageId, PublishError> {
        self.gsub.publish(topic, message)
    }

    /// Subscribe to new topic
    pub fn subscribe(&mut self, topic: IdentTopic) -> Result<bool, SubscriptionError> {
        self.gsub.subscribe(&topic)
    }

    // /// Get list of peers
    // pub fn peers(&mut self) -> &HashSet<PeerId> {
    //     self.kademlia
    // }
}