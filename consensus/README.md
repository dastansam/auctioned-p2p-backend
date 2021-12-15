# Smart Contracts

This directory contains all the necessary smart contracts of the project. I use Hardhat to easily compile, deploy and test smart contracts.

## Compile and Deploy

First, make sure to install dependencies:

```bash
yarn
```

To compile all contracts:
```bash
yarn run compile
```

Then, install the latest ganache-cli version globally:

```bash
npm install -g ganache-cli
```

Then run the ganache:

```
ganache-cli -d --mnemonic "ostrich seat balance wine together engine expand vapor claw immense friend empower" -b 5 -i 1337
```


This will run a local Ethereum test node at `7545` port. 

And now, deploy all contracts:

```bash
npx hardhat run scripts/deploy.js --network localhost
```

To interact with the whole platform, go to `frontend` directory and run the app:

```bash
cd ../frontend
yarn
yarn run start
```

## Auction Protocol

`AuctionProtocol` smart contract defines the classic time slot based auction. Validators, nodes that run the binary, bid for a certain slot and the node that is
willing to pay the most, will get to be the main processor for that slot.

Properties and rules of the protocol:

1. `AuctionProtocal` has a treasury address, which is responsible for collecting and refunding bids, and distributing commissions
2. Default node address and `nodeUrl` are defined as a fallback for slots without bids
3. `genesis` parameter defines the initial block number to start the slot auctions in the blockchain
4. Each slot is 40 blocks with each block having block time of 5 secs 
5. There are 40_000 slots
6. First two slots, slot #1 and slot #2 are not open for bidding.
7. For a slot X, if there are no bids, **default main processor** is selected as the validator
8. For a slot X, the deadline for bids is 20 blocks before the end of the slot. 
9. Nodes are encouraged to bid earlier, 40-50 blocks before, to make sure that their transactions are mined on time.

### Validator registration

To bid for a certain slot, validators need to be registered in the protocol. Users should execute `registerValidator` call supplying their `nodeUrl` in the 
peer-to-peer network. The url essentially is the `gRPC` endpoint of the backend, which clients would be able to interact with.

### Bid

Nodes can call `bid` to register their bid for a certain slot. 

