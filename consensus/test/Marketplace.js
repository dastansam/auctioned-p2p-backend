const hardhat = require('hardhat');
const { Contract, ethers, Wallet } = require('ethers');
const {expect} = require('chai');
const {
    Order, sign, createTypedData, 
    signedTypeData, Types, deployContracts
} = require('./_helper');

/**
Writes a simple match function
This function needs to accept as params:
All the raw params of the order commitment
(what NFT is sold - identified by contract and id, how much it is sold for - identified 
by ERC20 contract and amount, gossiper and anything else you've decided to include)
seller signature of keccak256 hash of all the params (when hashing use encode not encodePacked)
matcher signature of keccak256 hash of all the params
The flow of operations is going to be:

- The seller approves the marketplace contract to move their NFT (ERC721 approve)
- The seller signs the order commitment and sends it to the gossiping node
- The buyer requests the information = the matching node 
 (getting the matcher signature, allongside all the other info)
- The buyer approves the sell amount (ERC20 approve)
- The buyer sends a buy tx where
- The contract checks all the params and requirements
- The contract checks if the order commitment is signed by the seller
- The contract tries to transferFrom the NFT = the seller to the buyer 
- (might fail if the signer has sold it already or trasnferred it away)
- The contract tries to charge the buyer for the amount via transferFrom
- The contract deducts the commission fee that is distributed between the gossiper, 
  matcher and main processor (some/all of these might be the same addresses)
 */

describe("Marketplace", function() {
    this.timeout(200_000);

    const TOKEN_URI = "test-nft-uri";
    let deployedMarketplace = {};
    let deployedToken = {};
    let deployedNFT = {};
    let wallet = {};

    before(async () => {
        wallet = await hardhat.ethers.getSigners()[0];
        const { token, nft, marketplace } = await deployContracts();
        deployedMarketplace = marketplace;
        deployedToken = token;
        deployedNFT = nft;
    });

    async function defaultNFT() {
        return deployedNFT.mintNFT(wallet.address, TOKEN_URI);
    }

    async function getSignature(order, signer) {
		return sign(order, signer, deployedMarketplace.address);
	}

    it("should create an order commitment", async () => {
        // first give some erc20 tokens to buyer and seller
        const [owner, buyer, seller] = await hardhat.ethers.getSigners();
        await deployedToken.transfer(seller.address, 1_000_000);
        await deployedToken.transfer(buyer.address, 1_000_000);

        // - The seller approves the marketplace contract to move their NFT (ERC721 approve)
        // - The seller signs the order commitment
        const tokenId = await deployedNFT.mintNFT(seller.address, TOKEN_URI);
        await deployedNFT.connect(seller).approve(deployedMarketplace.address, '1');
        const sellOrder = Order(
            seller, 
            "0x0000000000000000000000000000000000000000", 
            deployedNFT.address,
            deployedToken.address,
            '1',
            owner.address,
            10_000,
            1, // SELL
        );

        const sellSignature = await getSignature(sellOrder, seller.address);

        //    - The buyer approves the sell amount (ERC20 approve)
        //    - The buyer sends a buy tx where
        console.log('approving erc20 tokens of buyer');
        await deployedToken.connect(buyer).approve(deployedMarketplace.address, 10_100);
        const buyOrder = Order(
            buyer,
            "0x0000000000000000000000000000000000000000",
            deployedNFT.address,
            deployedToken.address,
            tokenId,
            owner.address,
            10_000,
            0, // BUY
        );

        const buySignature = await getSignature(buyOrder, buyer.address);

        // call matchOrder call = the marketplace
        const result = await deployedMarketplace.connect(owner).matchOrder(
            buyOrder,
            buySignature,
            sellOrder,
            sellSignature,
            owner.address,
        );
        console.log(result);
    })

        // it("should be able to create a new NFT", async () => {
    //     await expect(defaultNFT())
    //         .to.emit(deployedToken, 'Transfer')
    //         .withArgs(ethers.constants.AddressZero, wallet.address, 1);
    // });

    // it("returns new item id", async () => {
    //     await expect(
    //         deployedNFT.mintNFT(wallet.address, TOKEN_URI)
    //     ).to.eq(1);
    // })
})