const { ethers } = require('ethers');
const hardhat = require('hardhat');
const DOMAIN_TYPE = [
    {
      type: "string",
      name: "name"
    },
    {
      type: "uint256",
      name: "chainId"
    },
  ];

// Deploy a contract
async function deploy(contractName, args) {
  const contract = await hardhat.ethers.getContractFactory(contractName);
  const deployed = await contract.deploy(...args);
  await deployed.deployed();
  return deployed;
}

// This is a script for deploying your contracts. You can adapt it to deploy
// yours, or create new ones.
async function deployContracts() {
    // This is just a convenience check
    if (network.name === "hardhat") {
      console.warn(
        "You are trying to deploy a contract to the Hardhat Network, which" +
          "gets automatically created and destroyed every time. Use the Hardhat" +
          " option '--network localhost'"
      );
    }

    // ethers is avaialble in the global scope
    const [deployer] = await hardhat.ethers.getSigners();
    console.log("Deployer:", deployer.address);
    console.log(
      "Deploying the contracts with the account:",
      await deployer.getAddress()
    );
  
    console.log("Account balance:", (await deployer.getBalance()).toString());
    
    // deploy Auction protocol
    const latestBlockNumber = await hardhat.ethers.provider.getBlockNumber();

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

    return {
        auction: auction,
        marketplace: marketplace,
        token: token,
        nft: nft
    }
}

function createTypedData(domainData, primaryType, message, types) {
    return {
        types: Object.assign({
            EIP712Domain: DOMAIN_TYPE,
        }, types),
        domain: domainData,
        primaryType: primaryType,
        message: message,
    };
}

function signedTypeData(provider, from, data) {
    return new Promise(async (resolve, reject) => {
        function callback(error, result) {
            if (error) {
                reject(error);
            } else {
                resolve(result);
            }
        }

        const signature = result.result;
        const signature0 = signature.substring(2);

        const r = "0x" + signature0.substring(0, 64);
        const s = "0x" + signature0.substring(64, 128);
        const v = parseInt(signature0.substring(128, 130), 16);

        resolve({
            data,
            signature,
            v, r, s
        });

        provider.sendAsync(
            {
                jsonrpc: "2.0",
                method: "eth_signTypedData_v3",
                params: [from, data],
                id: new Date().getTime(),
            },
            callback
        );
    });
}

function Order(signer, taker, contractAddress, tokenAddress, nftId, gossiper, price, orderType) {
	return { signer, taker, contractAddress, tokenAddress, nftId, gossiper, price, orderType };
}

const Types = {
	Order: [
		{name: 'signer', type: 'address'},
		{name: 'taker', type: 'address'},
		{name: 'contractAddress', type: 'address'},
		{name: 'tokenAddress', type: 'address'},
		{name: 'nftId', type: 'uint128'},
		{name: 'gossiper', type: 'address'},
		{name: 'price', type: 'uint128'},
		{name: 'orderType', type: 'u8'},
	]
};

async function sign(order, account, contract) {
	const provider = await ethers.providers.Web3Provider("http://localhost:8545");
	const chainId = Number(await provider.getChainId());

    const data = createTypedData({
		name: "Marketplace",
		chainId,
        contract
	}, 'Order', order, Types);
	return (await signedTypeData(provider, account, data)).sig;
}

module.exports={
    deployContracts,
    createTypedData,
    signedTypeData,
    Order,
    Types,
    sign
};