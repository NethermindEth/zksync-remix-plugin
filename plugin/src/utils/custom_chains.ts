import { defineChain } from 'viem'
 
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
        default: { name: 'Explorer', url: 'https://sepolia.explorer.zkcandy.io/' },
    },
    network: ''
})
