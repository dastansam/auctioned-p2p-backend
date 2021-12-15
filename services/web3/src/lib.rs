use async_std::channel::{Sender};
use ethers::core::rand::Rng;
use futures::{FutureExt, select, TryFutureExt};
use ethers::{prelude::*};
use common_types::{Error, NetworkMessage, Storage, AppStorage, OrderCommitment};
use std::convert::TryFrom;
use std::time::Duration;
use std::sync::Arc;

// Abi generation for contracts
abigen!(
    AuctionProtocol, 
    "./p2p/web3/abi/AuctionProtocol.json",
    event_derives(serde::Serialize, serde::Deserialize)
);
abigen!(
    Marketplace, 
    "./p2p/web3/abi/Marketplace.json",
    event_derives(serde::Serialize, serde::Deserialize)
);

/// Web3 Subscription service
pub struct Web3<DB: Storage> {
    // remote url for Ethereum node
    pub remote_url: String,
    // sender channel for sending messages to the network
    pub sender: Sender<NetworkMessage>,
    // provider for web3
    pub provider: Arc<Provider<Http>>,
    // wallet for signing transactions
    pub wallet: LocalWallet,
    // auction protocol address
    pub auction: ethers::types::Address,
    // marketplace contract address
    pub marketplace: ethers::types::Address,
    // additional topic to subscribe to
    pub topic: Option<ethers::types::H256>,
    // database
    pub db: Arc<DB>,
    // grpc address
    pub grpc_addr: String,
}

impl<DB> Web3<DB>
    where DB: AppStorage + Send + Sync + 'static
{
    pub async fn new(
        remote_url: String,
        sender: Sender<NetworkMessage>,
        wallet: LocalWallet,
        auction: ethers::types::Address,
        marketplace: ethers::types::Address,
        topic: Option<ethers::types::H256>,
        db: Arc<DB>,
        grpc_addr: String,
    ) -> Self {
        // intstantiate web3 provider
        let provider = Provider::<Http>::try_from(remote_url.to_owned())
            .unwrap()
            .interval(Duration::from_secs(5));

        let provider = Arc::new(provider);

        Web3 {
            remote_url,
            sender,
            provider,
            wallet,
            auction,
            marketplace,
            topic,
            db,
            grpc_addr
        }
    }

    /// Get provider
    pub async fn provider(&self) -> Arc<Provider<Http>> {
        self.provider.clone()
    }

    /// Marketplace contract
    pub async fn marketplace(&self) 
        -> Marketplace<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>
    {
        let provider = self.provider().await;
        let client = SignerMiddleware::new(
            provider.clone(), 
            self.wallet.clone()
        );
        let client = Arc::new(client);
        let marketplace = Marketplace::new(self.marketplace, client.clone());

        marketplace
    }

    /// Launches the subscription service.
    /// Tracks new blocks and new transactions.
    /// For each block, checks the current winner of the slot
    /// and gossips the message to the network
    /// Reacts to OrderMatch event from the marketplace contract
    /// and sends `Order remove` message to the network
    pub async fn launch_subscriptions(self) {
        let provider = self.provider().await;
        
        let client = SignerMiddleware::new(
            provider.clone(), 
            self.wallet.clone()
        );
        let client = Arc::new(client);

        // instantiate contracts
        let auction = AuctionProtocol::new(self.auction, client.clone());
        let marketplace = Marketplace::new(self.marketplace, client);

        // in the genesis, we will need to register our node in the Auction protocol
        // NOTE: this call should not be payable
        let register_tx = auction
            .method::<_, ()>(
                "registerValidator",
                (self.wallet.address().to_owned(), self.grpc_addr.clone()),
            )
            .unwrap()
            .legacy()
            .from(self.wallet.address().to_owned())
            .value(10000); // small fee for registration
        
        let pending_register = register_tx.send().await;

        match pending_register {
            Ok(tx_receipt) => {
                        // wait for two confirmations before doing anything
                let register_receipt = tx_receipt
                .await
                .unwrap_or_else(|e| {
                    println!("[WEB3] Error registering node: {:?}", e);
                    return None;
                });
    
                match register_receipt {
                    Some(_receipt) => {
                        println!("[WEB3] Registerd successfully: {:?}", self.wallet.address());
                    },
                    None => {
                        println!("[WEB3] Error registering: {:?}", register_receipt);
                    }
                };
            },
            Err(e) => {
                println!("[WEB3] Error registering node: {:?}", e);
            }
        }

        // Watch new blocks
        let mut block_stream = provider
            .watch_blocks()
            .await
            .unwrap();
        
        // Filter for auction events
        let auction_event = Filter::default()
            .address(ValueOrArray::Value(self.auction));
        
        // Filter for marketplace events
        let marketplace_event = Filter::default()
            .address(ValueOrArray::Value(self.marketplace));
        
        // stream that listens to auction protocol
        let mut auction_stream = provider
            .watch(&auction_event)
            .await
            .unwrap();
        
        // stream that listens to marketplace protocol
        let mut marketplace_stream = provider
            .watch(&marketplace_event)
            .await
            .unwrap();

        loop {
            select! {
                block = block_stream.next().fuse() => match block {
                    // new block received
                    // check if we are the winner
                    // if we are, send the message to the network
                    // if we are not, do nothing
                    Some(possible_header) => {
                        // block number
                        let number = provider
                            .get_block_number()
                            .await
                            .unwrap();
                        
                        // get current validator and node url from the auction protocol
                        let (node_url, address) = auction
                            .method::<_, (String, Address)>("getCurrentValidator", ())
                            .unwrap()
                            .call()
                            .await
                            .unwrap();

                        // get current slot from the auction protocol
                        let current_slot = auction
                            .method::<_, u128>("getCurrentSlotNumber", ())
                            .unwrap()
                            .call()
                            .await
                            .unwrap();

                        // send new message about current processor
                        if self.sender.send(NetworkMessage::CurrentProcessor{address}).await.is_err() {
                            println!("[WEB3] Error sending message to network");
                        };

                        // println!(
                        //     "[WEB3] New block {:?} received: {:?}", 
                        //     number, 
                        //     possible_header
                        // );

                        let stored_slot_number = self.db.slot_number();

                        // if the current slot is different from the stored slot number
                        // update the stored slot number
                        // and notify the network
                        if stored_slot_number < current_slot {
                            // send new message about current slot
                            if self.sender.send(NetworkMessage::NewSlot{
                                slot: current_slot,
                                address,
                            }).await.is_err() {
                                println!("Error sending message to network");
                            };

                            println!("New slot: {:?}", current_slot);
                            // we bid 3 slots ahead
                            let slot_to_bid: u16 = current_slot as u16 + 3;
                            let min_bid = auction
                                .method::<_, u128>("getMinBid", slot_to_bid)
                                .unwrap()
                                .call()
                                .await
                                .unwrap();
                            
                            // generate random bid by adding from 1 to 100 wei randomly
                            let random_bid = min_bid + rand::thread_rng().gen_range(1..100);
                            
                            // do the bid
                            let tx = auction
                                .method::<_, ()>("bid", (slot_to_bid, random_bid))
                                .unwrap()
                                .legacy()
                                .from(self.wallet.address().to_owned())
                                .value(random_bid + 10);
                            
                            // wait for confirmation
                            let pending_tx = tx
                                .send()
                                .await;
                            
                            match pending_tx {
                                Ok(tx_receipt) => {
                                    let receipt = tx_receipt
                                        .confirmations(2)
                                        .await
                                        .unwrap_or_else(|e| {
                                            println!("[WEB3] Error waiting for confirmation: {:?}", e);
                                            None
                                        });
                                
                                    match receipt {
                                        Some(_receipt) => {
                                            println!("[WEB3] Bid successful {}", random_bid);
                                        },
                                        None => {
                                            println!("[WEB3] Error sending bid");
                                        }
                                    };
                                },
                                Err(e) => {
                                    println!("[WEB3] Error bidding: {:?}", e);
                                }
                            };
                        }
                    },
                    // do nothing
                    None => {},
                },
                marketplace_event = marketplace_stream.next().fuse() => match marketplace_event {
                    // new event received
                    // check if it is an OrderMatch event
                    // if it is, send the message to the network
                    Some(raw_event) => {
                        println!("[WEB3] New event received: {:?}", raw_event.data);
                    },
                    None => {},
                },
                auction_event = auction_stream.next().fuse() => match auction_event {
                    // new event received
                    // check if it is an NewBid event
                    Some(raw_event) => {
                        println!("[WEB3] New auction event received: {:?}", raw_event.topics);
                    },
                    None => {},
                }
            }
        }
    }
}
