const hardhat = require('hardhat');
const { Contract, ethers, Wallet } = require('ethers');
const { expect } = require('chai');

const DEFAULT_VALIDATOR = "0x9fef2aa08cde1eb360c35600e14c453bdf8dcf7b";
const DEFAULT_URL = "http://127.0.0.1:50051";

describe('Auction Protocol tets', function(){
    this.timeout(50000);
    let deployedAuction = {};
    let owner = {};
    let bob = {};
    let alice = {};
    let dastan = {};

    // Deploy a contract
    async function deploy(contractName, args) {
        const contract = await hardhat.ethers.getContractFactory(contractName);
        const deployed = await contract.deploy(...args);
        await deployed.deployed();
        return deployed;
    }

    before(async () => {
        [owner, bob, alice, dastan] = await hardhat.ethers.getSigners();
        const latestBlockNumber = await hardhat.ethers.provider.getBlockNumber();
        deployedAuction = await deploy("AuctionProtocol", [
            "0x5542b9d2a0afc227f917eec349f1312fbe7c35cb",
            DEFAULT_VALIDATOR,
            DEFAULT_URL,
            latestBlockNumber + 1
        ]);
    })

    it('should deploy the contract', async () => {
        expect(deployedAuction.address).to.exist;
    });

    it('should be read state of the contract', async () => {
        const slot = await deployedAuction.getCurrentSlotNumber();
        expect(slot).to.eq(0);

        const [url, validator] = await deployedAuction.getCurrentValidator();
        expect(validator.toLowerCase()).to.eq(DEFAULT_VALIDATOR);
        expect(url).to.eq(DEFAULT_URL);
    })

    it("should check if node is ready for bidding", async function() {
        const isReady = await deployedAuction.isNodeRegistered(bob.address);

        expect(isReady).to.eq(false);

        const currentSlot = await deployedAuction.getCurrentSlotNumber();
        // check three slots ahead
        const minBid = await deployedAuction.getMinBid(currentSlot.add(3));

        expect(minBid).to.eq(0);
    })

    it(
        "should register node, check for minimum bid, participate in auction", 
        async function() {   
        const registerTx = await deployedAuction.connect(bob).registerValidator(bob.address, "bob-url");

        await registerTx.wait();

        const isReady = await deployedAuction.isNodeRegistered(bob.address);

        expect(isReady).to.eq(true);

        const currentSlot = await deployedAuction.getCurrentSlotNumber();
        // check three slots ahead
        const minBid = await deployedAuction.getMinBid(currentSlot.add(3));

        expect(minBid).to.eq(0);
        
        expect(deployedAuction.connect(bob).bid(
            currentSlot.add(5),
            minBid.add(99000),
            {
                value: 100_000
            }
        )).to.emit(deployedAuction, 'NewBid');
        
        expect(deployedAuction.connect(bob).bid(
            currentSlot.add(6),
            minBid.add(99000),
            {
                value: 100_000
            }
        )).to.emit(deployedAuction, 'NewValidator');
    })

    after(() => {
        process.exit(1);
    })
})