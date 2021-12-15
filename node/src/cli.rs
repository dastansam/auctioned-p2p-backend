use serde::Deserialize;
// use utils::get_home_dir;
use dirs::home_dir;
use structopt::StructOpt;
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    path::{PathBuf},
    io::{self, Write},
    cell::{RefCell},
};

use common_types::{Address};

/// Gets the home directory of the current user
pub fn get_home_dir() -> String {
    home_dir().unwrap().to_str().unwrap().to_owned()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub node_id: String,
    pub private_key: String,
    pub g_rpc_port: String,
    pub sync: bool,
    pub bootnode: Option<String>,
    pub eth_remote_url: String,
    pub auction_address: Address,
    pub marketplace_address: Address,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            node_id: "node01".to_string(),
            private_key: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            g_rpc_port: "50051".to_string(),
            sync: false,
            bootnode: None,
            // default Ganache port
            eth_remote_url: "http://127.0.0.1:8545".to_string(),
            auction_address: Address::zero(),
            marketplace_address: Address::zero()
        }
    }
}

#[derive(StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    pub options: CliOptions,
}

#[derive(StructOpt, Debug)]
pub struct CliOptions {
    #[structopt(short, long, help = "Given name for the node")]
    pub node_id: Option<String>,
    #[structopt(short, long, help = "Private key associated with the node")]
    pub private_key: String,
    #[structopt(short, long, help = "Port for gRPC server", )]
    pub g_rpc_port: Option<String>,
    #[structopt(short, long, help = "Sync with other nodes")]
    pub sync: bool,
    #[structopt(short, long, help = "Bootnode address")]
    pub bootnode: Option<String>,
    #[structopt(short, long, help = "Ethereum remote URL")]
    pub eth_remote_url: Option<String>,
    #[structopt(short, long, help = "Auction address")]
    pub auction_address: Option<String>,
    #[structopt(short, long, help = "Marketplace address")]
    pub marketplace_address: Option<String>,
}


impl CliOptions {
    pub fn to_config(&self) -> Result<Config, io::Error> {
        let mut config = Config::default();
        if let Some(node_id) = &self.node_id {
            config.node_id = node_id.to_string();
        }

        config.private_key = self.private_key.clone();

        if let Some(g_rpc_port) = &self.g_rpc_port {
            config.g_rpc_port = g_rpc_port.to_string();
        }
        config.sync = self.sync;
        config.bootnode = self.bootnode.clone();

        if let Some(eth_remote_url) = &self.eth_remote_url {
            config.eth_remote_url = eth_remote_url.to_string();
        }

        if let Some(auction_address) = &self.auction_address {
            config.auction_address = auction_address.parse::<Address>().unwrap();
        }

        if let Some(marketplace_address) = &self.marketplace_address {
            config.marketplace_address = marketplace_address.parse::<Address>().unwrap();
        }

        Ok(config)
    }
}