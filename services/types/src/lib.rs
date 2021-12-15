use rocksdb::DBIterator;
pub use rocksdb::{
    DB, Options, WriteBatch, 
    WriteOptions, IteratorMode,
};

pub use libp2p::{PeerId};
pub use libp2p::gossipsub::{IdentTopic};
pub use ethers::types::{Address};
pub use ethers::signers::{LocalWallet, Signer};
pub use uuid::Uuid;
pub use libp2p::kad::record::*;

use tonic::codec::{Decoder, Encoder};
use std::fmt::Display;
use async_std::channel::Sender;
use std::sync::Arc;
use std::io::{Cursor};
use thiserror::{Error};
use prost::Message;

pub mod node;

pub use node_rpc::{ OrderCommitment, OrderCommitmentList, EmptyRequest };
pub use node_rpc::node_rpc_server::{ NodeRpc, NodeRpcServer };

pub mod node_rpc {
    tonic::include_proto!("node_rpc");
}

/// Fallback main processor
pub const DEFAULT_MAIN_PROCESSOR: &str = "0x5542b9d2a0afc227f917eec349f1312fbe7c35cb";

#[derive(Debug, Clone)]
pub struct GRPCConfig {
    pub peer_id: PeerId,
    pub db: Arc<DB>,
    pub network_sender: Sender<NetworkMessage>,
    pub endpoint: String,
    pub address: Address,
}

#[derive(Debug)]
pub enum NetworkEvent {
    PubSubMessage {
        source: PeerId,
        message: Gossip,
    },
    PingRequest {
        source: PeerId,
    },
}


pub type SlotNumber = u128;

#[derive(Debug)]
pub enum Gossip {
    // List(Vec<String>),
    OrderCommitment(OrderCommitment),
    OrderCommitmentList(Vec<OrderCommitment>),
    Ping(),
}

#[derive(Debug)]
pub enum NetworkMessage {
    GossipMessage {
        source: PeerId,
        topic: IdentTopic,
        message: Vec<u8>
    },
    PingRequest {
        peer_id: PeerId,
    },
    NewOrderCommitment {
        source: PeerId,
        order_commitment: OrderCommitment,
    },
    RemoveOrder {
        id: String,
    },
    CurrentProcessor {
        address: Address,
    },
    NewSlot {
        address: Address,
        slot: u128,
    },
}



#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] rocksdb::Error),
    Other(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::Other(s) => write!(f, "Other error: {}", s)
        }
    }
}

impl From<Error> for String {
    fn from(err: Error) -> Self {
        err.to_string()
    }
}

/// Interface for Key-Value storage
pub trait Storage {
    /// Get a value from the storage
    fn read<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: AsRef<[u8]>;
    
    /// Put a value into the storage
    fn write<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>;
    
    /// Delete a value from the storage
    fn delete<K>(&self, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>;


    /// Check if a key exists in the storage
    fn contains<K>(&self, key: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]>;
    
    
    /// Create an iterator over the storage
    fn iterator(&self, mode: IteratorMode) -> DBIterator;
}

pub trait AppStorage: Storage {
    // Get typed object from the storage
    fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: Message + Default {
        let value = self.read(key)?;
        match value {
            Some(v) => {
                let decoded = T::decode(
                    &mut Cursor::new(v.as_slice())
                ).unwrap();
                Ok(Some(decoded))
            },
            None => Ok(None)
        }
    }

    /// Put typed object into the storage
    fn put<T>(&self, key: &str, value: &T) -> Result<(), Error>
    where
        T: Message + Default {
        let mut buff = Vec::new();
        buff.reserve(value.encoded_len());
        value.encode(&mut buff).unwrap();
        self.write(key, buff)
    }

    /// Put order commitment in the storage
    fn put_order_commitment(&self, order_commitment: OrderCommitment) -> Result<OrderCommitment, Box<Error>> {
        let key = format!("order_commitment_{}", order_commitment.order_id);
        let result = match self.put(&key, &order_commitment) {
            Ok(_) => {
                Ok(order_commitment)
            },
            Err(e) => {
                println!("Error writing order commitments: {}", e);
                Err(Box::new(e))
            }
        };
        result
    }

    /// Get all order commitments in the storage
    fn retrieve_order_commitments(&self) -> Result<OrderCommitmentList, Box<Error>> {
        let iter = self.iterator(IteratorMode::Start);
        let mut commitments = OrderCommitmentList::default();
        
        // iterate through the database and add all the order commitments to the list
        for (key, value) in iter {
            // check if b"order_commitment" is subset of key
            if key.starts_with(b"order_commitment") {
                let commitment = OrderCommitment::decode(
                    &mut Cursor::new(value.to_vec().as_slice())
                ).unwrap();
                commitments.order_commitments.push(commitment);
            }
        }

        Ok(commitments)
    }

    /// Delete order commitment from the storage
    fn delete_order_commitment(&self, order_id: &str) -> Result<String, Box<Error>> {
        let key = format!("order_commitment_{}", order_id);
        let result = match self.delete(&key) {
            Ok(_) => Ok(key),
            Err(e) => {
                println!("Error writing order commitments: {}", e);
                Err(Box::new(e))
            }
        };
        result
    }

    /// Get current slot number
    fn slot_number(&self) -> u128 {
        match self.read("slot_number") {
            Ok(Some(v)) => {
                let mut bytes: [u8; 16] = Default::default();
                bytes.copy_from_slice(&v);
                let slot_number = u128::from_be_bytes(bytes);
                slot_number
            },
            _ => {
                let slot_number: u128 = 0;
                self.write("slot_number", slot_number.to_be_bytes().as_slice())
                    .expect("Error writing slot number");
                
                slot_number
            }
        }
    }

    /// Set current slot number
    fn set_slot_number(&self, slot_number: u128) {
        self.write("slot_number", slot_number.to_be_bytes().as_slice())
            .expect("Error writing slot number");
    }

    /// Get node's address
    fn address(&self) -> Address {
        match self.read("address") {
            Ok(Some(v)) => {
                Address::from_slice(v.as_slice())
            },
            _ => {
                let address = Address::default();
                address
            }
        }
    }

    /// Get current main processor
    fn current_processor(&self) -> Address {
        match self.read("current_processor") {
            Ok(Some(v)) => {
                Address::from_slice(v.as_slice())
            },
            _ => {
                let address = DEFAULT_MAIN_PROCESSOR.parse::<Address>().unwrap_or(Default::default());
                address
            }
        }
    }
}