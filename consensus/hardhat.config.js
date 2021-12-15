require("@nomiclabs/hardhat-web3");
require("hardhat-spdx-license-identifier");
require("@nomiclabs/hardhat-waffle");

require("dotenv").config();

module.exports = {
    defaultNetwork: "hardhat",
    networks: {
        hardhat: {
            blockGasLimit: 8000000,
            allowUnlimitedContractSize: true,
            mining: {
                auto: false,
                interval: 5000,
            },
            accounts: {
                mnemonic: process.env.MAIN_MNEMONIC,
                path: process.env.DERIVATION_PATH,
                initialIndex: 0,
                count: 10,
            },
            // chainId: 1337
        },
        testnet: {
            url: process.env.ROPSTEN_URL,
            accounts: {
                mnemonic: process.env.MAIN_MNEMONIC,
                path: process.env.DERIVATION_PATH,
                initialIndex: 0,
                count: 20,
            }
        },
        ganache: {
            url: "http://127.0.0.1:8545"
        }
    },
    solidity: {
        version: "0.8.9",
        settings: {
            optimizer: {
                enabled: true,
                runs: 200,
            },
        },
    },
    spdxLicenseIdentifier: {
        overwrite: true,
        runOnCompile: true,
    },
    paths: {
        sources: "./contracts",
        tests: "./test"
    }
}
