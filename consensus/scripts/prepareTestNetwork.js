const path = require('path');

/**
 * (0) 0x5542B9d2A0aFc227f917eEC349f1312fbe7C35cB (100 ETH)
(1) 0x9fEf2AA08cde1EB360c35600E14C453BdF8DCf7B (100 ETH)
(2) 0xa9D93b8154A38e35Bd003294393cC29398825996 (100 ETH)
(3) 0xd45ffbF1260fbe1A2a6ED0F374C2b8c9740E23d0 (100 ETH)
(4) 0x9eAe81a7190227D74C5b3360439A0716C77667Fe (100 ETH)
(5) 0xd8a37002e4a9d2960b623234e2Fe67021243b9bd (100 ETH)
(6) 0xAbd2C3209d909E62cE510EacbDB20A131E071EbE (100 ETH)
(7) 0x79ab91083ba4A24fF58CFf278106D3D218A80F77 (100 ETH)
(8) 0xb3710a9d45Aee740a66Eaf2F0aa442606e686209 (100 ETH)
(9) 0xF222F1678BD498B4FCA731fB77A4189e8726a22d
 */

const addresses = [
    "0x9fEf2AA08cde1EB360c35600E14C453BdF8DCf7B",
    "0xa9D93b8154A38e35Bd003294393cC29398825996",
    "0xd45ffbF1260fbe1A2a6ED0F374C2b8c9740E23d0",
    "0x9eAe81a7190227D74C5b3360439A0716C77667Fe",
    "0xd8a37002e4a9d2960b623234e2Fe67021243b9bd",
    "0xAbd2C3209d909E62cE510EacbDB20A131E071EbE",
    "0x79ab91083ba4A24fF58CFf278106D3D218A80F77",
    "0xb3710a9d45Aee740a66Eaf2F0aa442606e686209",
    "0xF222F1678BD498B4FCA731fB77A4189e8726a22d"
];


async function prepareTestNetwork() {
    const [deployer] = await ethers.getSigners();

    const nftArtifact = artifacts.readArtifactSync("TestNFT");
    const nftContract = new ethers.Contract(
        "0x3B92d83A02465F52F80d1265aBaDeF29056dcB48",
        nftArtifact.abi,
        deployer
    );
    
    const erc20Artifact = artifacts.readArtifactSync("WrappedUSD");
    const erc20Contract = new ethers.Contract(
        "0x2B31AF0a19c5a01a0ca5A300b85977aFE8bf4acA",
        erc20Artifact.abi,
        deployer
    );

    for (address of addresses) {
        await nftContract.mintNFT(address, "URI:" + address.slice(2, 12));
        console.log(`Minted an NFT for ${address}`);
        console.log(`Minting ${10000} USD for ${address}`);
        await erc20Contract.transfer(address, 10000000);
    }
}

prepareTestNetwork()
    .then(() => {
        console.log("Test network prepared");
        process.exit(0);
    }
).catch(console.error);
