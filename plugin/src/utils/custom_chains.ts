import {defineChain} from 'viem'

export const zkCandySepoliaTestnet = defineChain({
    id: 302,
    name: 'zkCandy Sepolia Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://sepolia.rpc.zkcandy.io/'],
        },
        public: {
            http: ['https://sepolia.rpc.zkcandy.io/'],
        }
    },
    blockExplorers: {
        default: {name: 'Explorer', url: 'https://sepolia.explorer.zkcandy.io/'},
    },
    network: ''
})

export const zkLinkNova = defineChain({
    id: 810180,
    name: 'zkLink Nova',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.zklink.io'],
            webSocket: ['wss://rpc.zklink.io']
        },
        public: {
            http: ['https://rpc.zklink.io'],
            webSocket: ['wss://rpc.zklink.io']
        }
    },
    blockExplorers: {
        default: {name: 'Explorer', url: 'https://explorer.zklink.io'},
    },
    network: '',
    contracts: {
        multicall3: {address: '0x825267E0fA5CAe92F98540828a54198dcB3Eaeb5'}
    }
})

export const zkLinkNovaTestnet = defineChain({
    id: 810181,
    name: 'zkLink Nova Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://sepolia.rpc.zklink.io'],
            webSocket: ['wss://sepolia.rpc.zklink.io']
        },
        public: {
            http: ['https://sepolia.rpc.zklink.io'],
            webSocket: ['wss://sepolia.rpc.zklink.io']
        }
    },
    blockExplorers: {
        default: {name: 'Explorer', url: 'https://sepolia.explorer.zklink.io'},
    },
    network: ''
})

export const sophonTestnet = defineChain({
    id: 531050104,
    name: 'Sophon testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Sophon',
        symbol: 'SOPH',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.sophon.xyz'],
            webSocket: ['wss://rpc.testnet.sophon.xyz']
        },
        public: {
            http: ['https://rpc.testnet.sophon.xyz'],
            webSocket: ['wss://rpc.testnet.sophon.xyz']
        }
    },
    blockExplorers: {
        default: {name: 'Explorer', url: 'https://explorer.testnet.sophon.xyz'},
    },
    network: ''
})