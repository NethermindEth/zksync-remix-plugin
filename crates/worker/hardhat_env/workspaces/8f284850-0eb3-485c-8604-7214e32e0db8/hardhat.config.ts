
import { HardhatUserConfig } from "hardhat/config";

import "@matterlabs/hardhat-zksync-solc";
import "@matterlabs/hardhat-zksync-verify";

export const zkSyncTestnet = process.env.NODE_ENV == "test"
? {
    url: "http://127.0.0.1:8011",
    ethNetwork: "http://127.0.0.1:8045",
    zksync: true,
  }
: {
    url: "https://sepolia.era.zksync.dev",
    ethNetwork: "sepolia",
    zksync: true,
    verifyURL: "https://explorer.sepolia.era.zksync.dev/contract_verification"
  };

export const zkSyncMainnet = {
    url: "https://mainnet.era.zksync.io",
    ethNetwork: "mainnet",
    zksync: true,
    verifyURL: "https://zksync2-mainnet-explorer.zksync.io/contract_verification"
  };

const config: HardhatUserConfig = {
  zksolc: {
    version: "1.4.1",
    settings: {},
  },
  defaultNetwork: "zkSyncTestnet",
  networks: {
    hardhat: {
      zksync: false,
    },
    zkSyncTestnet,
    zkSyncMainnet,
  },
  solidity: {
    version: "0.8.24",
  },
  paths: {sources: "./contracts",},
};

export default config;
