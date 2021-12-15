use std::io::{Cursor};
use std::sync::Arc;
use std::fmt::LowerHex;
use std::error::Error;
use async_std::channel::Sender;
use libp2p::PeerId;
use prost::Message;
// use node::service::{DB};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use common_types::{
    AppStorage, EmptyRequest, 
    Error as DBError, NetworkMessage, 
    NodeRpc, NodeRpcServer, OrderCommitment, 
    OrderCommitmentList, Storage, Address,
    Uuid
};

/// Starts a gRPC server that listens on the specified port.
pub async fn start_g_rpc<DB: Storage> (
    peer_id: PeerId,
    db: Arc<DB>,
    network_sender: Sender<NetworkMessage>,
    endpoint: &str,
    address: Address,
) -> Result<(), Box<dyn Error + Send + Sync>> 
    where 
        DB: AppStorage + Send + Sync + 'static
    {
        let addr = endpoint.parse().unwrap();
        let service = GRPCService{peer_id, db, network_sender, address};

        println!("[GRPC] Ready on http://{}", addr);

        let svc = NodeRpcServer::new(service);
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await?;
        
        println!("[GRPC] Shutting down the server...");
        
        Ok(())
}

#[derive(Debug, Clone)]
pub struct GRPCService<DB> {
    peer_id: PeerId,
    pub address: Address,
    pub db: Arc<DB>,
    network_sender: Sender<NetworkMessage>
}

#[tonic::async_trait]
impl<DB> NodeRpc for GRPCService<DB>
    where DB: AppStorage + Send + Sync + 'static
{
    async fn ping(&self, request: Request<EmptyRequest>) -> Result<Response<EmptyRequest>, Status> {
        if self.network_sender.send(NetworkMessage::PingRequest{peer_id: PeerId::random()}).await.is_err() {
            println!("[GRPC] Error sending ping request");
        };

        Ok(Response::new(EmptyRequest::default()))
    }

    /// Get stored order commitments from the storage
    async fn get_order_commitments(&self, request: Request<EmptyRequest>) -> Result<Response<OrderCommitmentList>, Status> {
        let commitments = match self.db.retrieve_order_commitments() {
            Ok(commitments) => commitments,
            Err(e) => {
                println!("[GRPC] Error retrieving order commitments: {}", e);
                return Err(Status::new(
                    tonic::Code::Internal,
                    format!("[GRPC] Error retrieving order commitments: {}", e)
                ));
            }
        };

        Ok(Response::new(commitments))
    }

    /// Cancel order commitment
    async fn cancel_order_commitment(&self, request: Request<OrderCommitment>) -> Result<Response<EmptyRequest>, Status> {
        println!("cancel_order_commitment: {:?}", request);
        let commitment = request.into_inner();

        match self.db.delete_order_commitment(&commitment.order_id) {
            Ok(id) => {
                if self.network_sender.send(
                    NetworkMessage::RemoveOrder {
                        id
                    }
                ).await.is_err()
                {
                    println!("[GRPC] Order doesn't exist");
                };
                Ok(Response::new(EmptyRequest::default()))
            },
            Err(e) => {
                println!("[GRPC] Error creating order commitment: {}", e);
                Err(Status::new(
                    tonic::Code::Internal,
                    format!("[GRPC] Error creating order commitment: {}", e)
                ))
            }
        }
    }

    /// Create order commiment and send it to the network
    /// First stores it in the
    async fn create_order_commitment(&self, request: Request<OrderCommitment>) -> Result<Response<OrderCommitment>, Status> {
        let mut commitment = request.into_inner();
        match self.db.put_order_commitment(commitment.clone()) {
            Ok(_) => {
                commitment.gossiper = self.address.to_string();

                // gossip about new order commitment
                if self.network_sender.send(
                        NetworkMessage::NewOrderCommitment {
                            order_commitment: commitment.clone(),
                            source: self.peer_id.clone()
                        }
                    ).await.is_err()
                {
                    println!("[GRPC] Error gossiping order commitment");
                };
                Ok(Response::new(commitment))
            },
            Err(e) => {
                println!("[GRPC] Error creating order commitment: {}", e);
                Err(Status::new(
                    tonic::Code::Internal,
                    format!("[GRPC] Error creating order commitment: {}", e)
                ))
            }
        }
    }
}

impl<DB> GRPCService<DB> where DB: AppStorage + Send + Sync + 'static {
    pub fn new(
        peer_id: PeerId, 
        db: Arc<DB>, 
        network_sender: Sender<NetworkMessage>,
        address: Address
    ) -> Self {
        GRPCService {peer_id, db, network_sender, address}
    }

    /// Get key value store.
    pub fn get_storage(&self) -> &DB {
        &self.db
    }
}
