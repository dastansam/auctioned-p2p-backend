marketplace = 0xdC937D0CfdE76144bcfc1336630Ad8854566C2F4
auction = 0x1536383CE8E7c70fCb8A4AFF2275E0c7D71D7519
private_key1 = 3014cc8374696c3efbe2617c20d591751f3bb3b6d30df32506b729928b58b836
private_key2 = 9221bd3e2a1ccc039b6f7779c26b3a60560421641f1a550d0767e09ace8a77fb
private_key3 = 6070a5a5651492a41340b7c4ba2cfbd9d2236d4e1662081448d54ed97e3a0aff

all: test build network node01 node02 node03

# before tests, we purge the database
test:
	yarn --cwd=./consensus test:network

test-contracts:
	yarn --cwd=./consensus test:auction

# spawn network
network: node02 node01 node03

# spawn one node
node01: purge
	./exe/nft-node -m $(marketplace) -a $(auction) -p $(private_key1) -n node01 -g 50051 &> /dev/null

# spawn second node
node02: purge
	./exe/nft-node -m $(marketplace) -a $(auction) -p $(private_key2) -n node02 -g 50052 &> /dev/null 

# spawn third node
node03: purge
	./exe/nft-node -m $(marketplace) -a $(auction) -p $(private_key3) -n node03 -g 50053 &> /dev/null 

# build the node
build:
	cargo build --release

purge:
	bash purge-db.sh

ganache:
	cd consensus && ganache-cli -d --mnemonic "ostrich seat balance wine together engine expand vapor claw immense friend empower" -b 5 -i 1337

deploy:
	cd consensus && npx hardhat run ./scripts/deploy.js --network ganache

frontend:
	cd frontend && node ./server.js &> /dev/null && yarn start

