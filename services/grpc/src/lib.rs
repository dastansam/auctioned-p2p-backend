use std::error::Error;
use std::task::Poll;
use futures::stream::FusedStream;
use futures::{Future, Stream};
use tonic::codegen::Service;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use nft_node::{ OrderCommitment, OrderCommitmentList };
use nft_node::node_rpc::{ NodeRpc, NodeRpcServer };

pub mod nft_node {
    tonic::include_proto!("node_rpc");
}

/// Starts a gRPC server that listens on the specified port.
pub async fn start_gRPC<DB> (
    db: Arc<DB>,
    endpoint: &str
) -> Result<Response<(), Box<dyn Error>>> {
    let addr = endpoint.parse().unwrap();
    let service = GRPCService::new(db);

    println!("Ready on http://{}", addr);

    let server = Server::builder()
        .add_service(NodeRpcServer::new(service))
        .serve(addr)
        .await?;
    
    println!("Shutting down the server...");
    Ok(())
}

#[derive(Debug, Clone)]
pub struct GRPCService<DB> {
    db: Arc<DB>,
}

#[tonic::async_trait]
impl NodeRpc for GRPCService {
    async fn create_order(
        &self,
        request: Request<OrderCommitment>
    ) -> Result<Response<OrderCommitment>, Status> {
        println!("commit_order: {:?}", request);
        Ok(Response::new(()))
    }

    async fn ping(&self) -> Result<Response<()>, Status> {
        println!("ping");
        Ok(Response::new(()))
    }

    async fn get_order_commitments(&self) -> Result<Response<OrderCommitmentList>, Status> {
        println!("get_order_commitments");
        Ok(Response::new(vec![]))
    }
}
