import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import { useEffect } from 'react'
import reportWebVitals from './reportWebVitals'
import "@rainbow-me/rainbowkit/styles.css";
import { getDefaultWallets, RainbowKitProvider, darkTheme, Chain} from "@rainbow-me/rainbowkit";

import { configureChains, createClient, WagmiConfig } from "wagmi";
import { zkSync, mainnet, zkSyncTestnet, goerli } from "wagmi/chains";
import { publicProvider } from "wagmi/providers/public";



// declare const zkSyncMainnnet: {
//   readonly id: 324;
//   readonly name: "zkSync Era";
//   readonly network: "zksync-era";
//   readonly nativeCurrency: {
//       readonly decimals: 18;
//       readonly name: "Ether";
//       readonly symbol: "ETH";
//   };
//   readonly rpcUrls: {
//       readonly default: {
//           readonly http: readonly ["https://mainnet.era.zksync.io"];
//           readonly webSocket: readonly ["wss://mainnet.era.zksync.io/ws"];
//       };
//       readonly public: {
//           readonly http: readonly ["https://mainnet.era.zksync.io"];
//           readonly webSocket: readonly ["wss://mainnet.era.zksync.io/ws"];
//       };
//   };
//   readonly blockExplorers: {
//       readonly default: {
//           readonly name: "zkExplorer";
//           readonly url: "https://explorer.zksync.io";
//       };
//   };
//   readonly contracts: {
//       readonly multicall3: {
//           readonly address: "0x47898B2C52C957663aE9AB46922dCec150a2272c";
//       };
//   };
// };


// const zksync: Chain = {
//   id: 280,
//   name: 'zkSync Era Testnet',
//   network: 'zkSync Era Testnet',
//   iconUrl: 'https://example.com/icon.svg',
//   iconBackground: '#fff',
//   nativeCurrency: {
//     decimals: 18,
//     name: 'Ethereum',
//     symbol: 'ETH',
//   },
//   rpcUrls: {
//     default: {
//       http:["https://testnet.era.zksync.dev"],
//       webSocket: ["wss://testnet.era.zksync.dev/ws"]
//     },
//     public: { http: ['https://testnet.era.zksync.dev'],
//               webSocket: ["wss://testnet.era.zksync.dev/ws"]
//     }
//   },
//   blockExplorers: {
//     default: { name: 'zkSync Era Testnet Explorer', url: 'https://goerli.explorer.zksync.io/' },
//   },
//   testnet: true,
// };

const { chains, provider, webSocketProvider } = configureChains(
  [
    zkSync,
    mainnet,
    zkSyncTestnet,
    goerli,
    // ...(process.env.NEXT_PUBLIC_ENABLE_TESTNETS === "true"
    //   ? [zkSyncTestnet, goerli]
    //   : []),
  ],
  [publicProvider()]
);

const { connectors } = getDefaultWallets({
  appName: "zkSync Greeter",
   projectId: "96384f1cfd105651ddb35f0d0d70b198", // needed for WalletConnect -- get from https://cloud.walletconnect.com/
  chains,
});

const wagmiClient = createClient({
  autoConnect:true,
  connectors,
  provider,
  webSocketProvider,
});

// const wagmiConfigWalletConnect = defaultWagmiConfig({ chains, projectId, metadata })
// // 3. Create modal
//createWeb3Modal({ wagmiConfigWalletConnect, projectId, chains })


// useEffect(() => {
//   if (window.ethereum) {
//     window.ethereum.on('accountsChanged', function (accounts) {
//       // Inform RainbowKit of the account change or force a component re-render.
//       console.log('accountsChanged   woooooohooooooo')

//     });
//   }
//   else {
//     console.warn('No Ethereum object detected. Install MetaMask or another Ethereum wallet provider.');
//   }
// }, []);
const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)

root.render(
  <WagmiConfig client={wagmiClient}>
  <RainbowKitProvider
    chains={chains}
    initialChain={zkSyncTestnet}
    modalSize="compact"
    showRecentTransactions={true}
  >

      <App />
     </RainbowKitProvider>
   </WagmiConfig>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
