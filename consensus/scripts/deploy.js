const path = require("path");

// This is a script for deploying your contracts. You can adapt it to deploy
// yours, or create new ones.
async function main() {
    // This is just a convenience check
    if (network.name === "hardhat") {
      console.warn(
        "You are trying to deploy a contract to the Hardhat Network, which" +
          "gets automatically created and destroyed every time. Use the Hardhat" +
          " option '--network localhost'"
      );
    }

    // ethers is avaialble in the global scope
    const [deployer] = await ethers.getSigners();
    console.log(
      "Deploying the contracts with the account:",
      await deployer.getAddress()
    );
  
    console.log("Account balance:", (await deployer.getBalance()).toString());
    
    // deploy Auction protocol
    const latestBlockNumber = await ethers.provider.getBlockNumber();

    const auction = await deploy("AuctionProtocol", [
      "0x5542b9d2a0afc227f917eec349f1312fbe7c35cb",
      "0x9fef2aa08cde1eb360c35600e14c453bdf8dcf7b",
      "/ip4/p2p/nft/test-peer-id",
      latestBlockNumber + 10
    ]);

    // deploy marketplace
    const marketplace = await deploy("Marketplace", [
      [1, 10, 89],
      "0x5542b9d2a0afc227f917eec349f1312fbe7c35cb"
    ]);

    // deploy token
    const token = await deploy("WrappedUSD", []);

    // deploy nft contract
    const nft = await deploy("TestNFT", []);

    console.log("Protocol address:", auction.address);
    console.log("Marketplace address:", marketplace.address);
    console.log("WrappedUSD address:", token.address);
    console.log("TestNFT address:", nft.address);

    //We also save the contract's artifacts and address in the frontend directory
    saveFrontendFiles(auction);
    saveFrontendFiles(marketplace);
    saveFrontendFiles(token);
    saveFrontendFiles(nft);
  }
  
function saveFrontendFiles(token) {
  const fs = require("fs");
  const contractsDir = path.join(__dirname, "/../../frontend/src/contracts");

  if (!fs.existsSync(contractsDir)) {
    fs.mkdirSync(contractsDir);
  }

  const AuctionArtifact = artifacts.readArtifactSync("AuctionProtocol");
  const MarketplaceArtifact = artifacts.readArtifactSync("Marketplace");

  fs.writeFileSync(
    contractsDir + "/AuctionProtocol.json",
    JSON.stringify(AuctionArtifact, null, 2)
  );
  fs.writeFileSync(
    contractsDir + "/Marketplace.json",
    JSON.stringify(MarketplaceArtifact, null, 2)
  );
}

// Deploy a contract
async function deploy(contractName, args) {
  const contract = await ethers.getContractFactory(contractName);
  const deployed = await contract.deploy(...args);
  await deployed.deployed();
  return deployed;
}
  
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });