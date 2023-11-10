// import { HardhatUserConfig } from "hardhat/config";
// import "@matterlabs/hardhat-zksync-deploy";
// import "@matterlabs/hardhat-zksync-solc";
// import "@matterlabs/hardhat-zksync-verify";
//
// const config: HardhatUserConfig = {
//   zksolc: {
//     version: "1.3.14",
//     settings: {},
//   },
//   solidity: {
//     version: "0.8.17",
//   },
//   paths: {
//     sources: "./contracts/abc",
//   }
// };
//
// export default config;

module.exports = {
  zksolc: {
    version: "1.3.12",
    settings: {},
  },
  solidity: {
    version: "0.8.17",
  },
  paths: {
    sources: "./contracts/abc",
  }
};
