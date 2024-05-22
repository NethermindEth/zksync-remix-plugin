import '@matterlabs/hardhat-zksync-solc';

import { HardhatUserConfig } from 'hardhat/config';

const config: HardhatUserConfig = {
    zksolc: {
        compilerSource: 'binary',
        settings: {
            isSystem: true,
            optimizer: {
                enabled: true,
            },
            compilerPath: '/usr/local/bin/zksolc'
        }
    },
    defaultNetwork: 'dockerizedNode',
    networks: {
        hardhat: {
            zksync: true,
        },
        dockerizedNode: {
            url: "http://0.0.0.0:3050",
            ethNetwork: "http://0.0.0.0:8545",
            zksync: true,
        },
    },
    solidity: {
        version: '0.8.17',
    },
};

export default config;