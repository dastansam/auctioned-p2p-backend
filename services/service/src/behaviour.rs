use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Poll, Context};
use std::time::Duration;

use async_std::task;
use db::rocks::RocksDB;
use libp2p::identity::Keypair;
use std::io::{Cursor};
use libp2p::{PeerId};
use libp2p::gossipsub::error::{PublishError, SubscriptionError};
use libp2p::kad::{AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryId, QueryResult, Record, Quorum,};
use libp2p::gossipsub::{self, Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic, MessageAuthenticity, MessageId, Topic};

use libp2p::mdns::MdnsConfig;
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};
use libp2p::{
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, toggle::Toggle},
    NetworkBehaviour,
};
use prost::Message;
use common_types::{OrderCommitment, AppStorage};

use libp2p::kad::record::store::MemoryStore;

// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub(crate) struct NodeBehaviour {
    pub gsub: Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    peers: Vec<PeerId>,
    #[behaviour(ignore)]
    db: Arc<RocksDB>,
}

impl NetworkBehaviourEventProcess<MdnsEvent> for NodeBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(list) = event {
            for (peer_id, multiaddr) in list {
                println!("[MDNS] Discovered peer {:?} {:?}", peer_id, multiaddr);
                self.gsub.add_explicit_peer(&peer_id);
                self.kademlia.add_address(&peer_id, multiaddr);
                self.peers.push(peer_id);
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
                let topic = message.topic.to_string();
                println!("{}", topic);
                if topic.to_string() == "order_commitment" {
                    let order_commitment = OrderCommitment::decode(
                        Cursor::new(message.data.to_vec())
                    );
                    match order_commitment {
                        Ok(order_commitment) => {
                            if self.db.put_order_commitment(order_commitment).is_err() {
                                println!("[GoSSIPSUB] Couldn't store order commitment in db");
                            };
                        },
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                else if topic.to_string() == "cancel_order" {
                    let order_id = String::from_utf8(message.data.to_vec());
                    match order_id {
                        Ok(id) => {
                            if self.db.delete_order_commitment(&id).is_err() {
                                println!("[GoSSIPSUB] Couldn't delete order commitment in db");
                            };
                        },
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
            },
            GossipsubEvent::Subscribed{
                peer_id,
                topic
            } => {
                println!("[GOSSIPSUB] Connected to topic: {:?}", topic);
            }
            GossipsubEvent::Unsubscribed{
                peer_id,
                topic
            } => {
                println!("[GOSSIPSUB] Disconnected from peer");
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for NodeBehaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            _ => ()
        }
    }
}

impl NodeBehaviour {
    pub fn new(
        local_key: &Keypair, 
        peer_id: PeerId, 
        bootnode: Option<String>,
        db: Arc<RocksDB>,
    ) -> Self {
        // create message id function for gossipsub
        let message_id_fn = |message: &GossipsubMessage| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        // Gossipsub configuration
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::None)
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
        
        let mut behaviour = NodeBehaviour {
            kademlia, mdns, 
            gsub: gossipsub,
            peers: Vec::new(),
            db
        };

        // create topic for subscription
        let order_topic = IdentTopic::new("order_commitment");
        let cancel_order = IdentTopic::new("cancel_order");
        let ping_topic = IdentTopic::new("ping");

        // subscribe to node topic
        behaviour.gsub.subscribe(&order_topic).unwrap();
        behaviour.gsub.subscribe(&cancel_order).unwrap();
        behaviour.gsub.subscribe(&ping_topic).unwrap();
        
        if let Some(bootnode) = bootnode {
            let peer_id = PeerId::from_str(&bootnode).expect("invalid peer id");
            behaviour.gsub.add_explicit_peer(&peer_id);
        };

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

    /// Get list of peers
    pub fn peers(&self) -> Vec<PeerId> {
        self.peers.clone()
    }
}