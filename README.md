# Decentralized Marketplace of Backends

Auctioned marketplace backend repository. Defines the peer-to-peer node and it's logic.

## Introduction

This is a POC project that explores the realm of *managed decentralization*. By running this node, users launch the instance of the NFT (or any) marketplace backend that serves the requests of users. NFT items on sale are stored as `OrderCommitment` (see specs below) and for each order commitment gossiped to other peers, or order matched by the node, node gets the commission from the sale.

## Test private keys

To test the nodes, you will need to at least supply private key. I have a test `mnemonic` seed that can generate an HD wallet, i.e can be used to derive infinite number of private keys.

Here is the list of 10 private and public keys you can supply as an argument to the node.

```
Available Accounts
==================
(0) 0x5542B9d2A0aFc227f917eEC349f1312fbe7C35cB (100 ETH)
(1) 0x9fEf2AA08cde1EB360c35600E14C453BdF8DCf7B (100 ETH)
(2) 0xa9D93b8154A38e35Bd003294393cC29398825996 (100 ETH)
(3) 0xd45ffbF1260fbe1A2a6ED0F374C2b8c9740E23d0 (100 ETH)
(4) 0x9eAe81a7190227D74C5b3360439A0716C77667Fe (100 ETH)
(5) 0xd8a37002e4a9d2960b623234e2Fe67021243b9bd (100 ETH)
(6) 0xAbd2C3209d909E62cE510EacbDB20A131E071EbE (100 ETH)
(7) 0x79ab91083ba4A24fF58CFf278106D3D218A80F77 (100 ETH)
(8) 0xb3710a9d45Aee740a66Eaf2F0aa442606e686209 (100 ETH)
(9) 0xF222F1678BD498B4FCA731fB77A4189e8726a22d (100 ETH)

Private Keys
==================
(0) 0x1e51caae3182b5f138216b9324f24833c4134d2946030ce8df5cca20e9591f1e
(1) 0x3014cc8374696c3efbe2617c20d591751f3bb3b6d30df32506b729928b58b836
(2) 0x9221bd3e2a1ccc039b6f7779c26b3a60560421641f1a550d0767e09ace8a77fb
(3) 0x6070a5a5651492a41340b7c4ba2cfbd9d2236d4e1662081448d54ed97e3a0aff
(4) 0x654a23005daa65949a117d2f4f0cd87106304e0e86b26308b879f96f517f6f53
(5) 0x2bc9e17ca851874412388eec41c17057bbbc0e31be9dfe8eddb4966f01f18517
(6) 0xa5ae80c95ddb7f2b5b82b68c445177300904cf29c7b54a820ee0610c0f125346
(7) 0x5489199439055b363ec20d9ac1d41d1fab16741524fa78fb9663bed512fc5534
(8) 0x2f58d6d2ff6b1f05a0375c61530a5594fb4249249dfc0978c2db0cac3d08f957
(9) 0x977206021a17b5dcd8ac3e6505e3d42a08ba7e1a4893a695211434751632d498
```

Alternatively, you can generate them when you run `make ganache` command.

## Install and run
To install and run the node, make sure you have the essentials of Rust project development are installed. Clone this repository.

Please, make sure to install all the npm dependencies before doing steps below:

```
npm install -g ganache-cli
cd consensus && yarn
cd frontend && yarn
```

After that make sure to run the ganache node and deploy contracts:

```
make ganache
```

And in anoter terminal window:

```
make deploy
```

To run the node quickly, you can execute the binary in the `exe` folder. 

```
./exe/nft-node \
-m 0xdC937D0CfdE76144bcfc1336630Ad8854566C2F4 \
-a 0x1536383CE8E7c70fCb8A4AFF2275E0c7D71D7519 \
-p 3014cc8374696c3efbe2617c20d591751f3bb3b6d30df32506b729928b58b836 \
-n node01 \
-g 50051
```

If you have difficulty running the above command, try to eliminate redundant whitespaces or other symbols.

### Build from scratch

To compile and run the node:

```
cargo run -- -p <private-key> -m <marketplace address> -a <auction-address>
```

This will run the default node and launch the gRPC service at the default port (50051).

Or to build a release version of the node:

```bash
cargo build --release
# OR
make build
```

This command will generate an executable binary that can be launched:

```
./target/release/nft-node -p <private-key> -m <marketplace address> -a <auction-address> -n <name> -g <grpc-port>
```

If you want to run another instance of the node, launch another terminal in the same directory and run the code below, replacing it with your values:

```
cargo run -- -n <Name of the node> -g <Port number for the gRPC> -p <private-key> -a <auction address> -m <marketplace address>
```

It is important to supply unique arguments for `-n` and `-g`, otherwise the node will fail to launch.

You should now see that both nodes detect each other and add each other in their peers list.

To test the gRPC service, first install `grpcurl` package:

```zsh
brew install grpcurl
```
Then make a simple ping request to one of the nodes and see that the ping message is broadcasted:

```zsh
grpcurl -plaintext -import-path ./p2p/types/proto -proto node_rpc.proto -d '{}' [::]:50051 node_rpc.NodeRpc/ping
```

## Tests
Make sure to purge the db before tests:

For all the tests, you need to have a ganache node running locally.

Integration tests can be run like this:

```
make -j4 node01 node02 node03 test
```

And unit tests

```
make test-contracts
```

## Services

The project consists of three integral services:

- `p2p_service`
    - This package is responsible for p2p network logic. Specifically, it deals with node syncing, discovering and gossiping. Essentially, this defines the business logic of the node.
    - `Kademlia` protocol is used for peer discovery, memory storage, etc.
    - `GossipSub` protocol is used for gossiping messages in the network
    - `mDNS` protocol is used to discover peers in the same network (used for small networks)
    - There are also `peers`, `events` properties of the p2p network logic, that are used to store connected peers and the events produced by the node, respectivelly.
- `grpc`
    - `gRPC` crate defines the corresponding gRPC service that serves the user requests.
- `DB`
    - Defines the necessary traits for the Storage and setup for the `RocksDB` key-value storage.
- `common-types`
    - This crate contains the common types used in the project.
