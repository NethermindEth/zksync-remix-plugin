import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import { createWeb3Modal, defaultWagmiConfig } from '@web3modal/wagmi/react'
import { configureChains, createConfig, WagmiConfig } from 'wagmi';
import { publicProvider } from 'wagmi/providers/public';
import { CoinbaseWalletConnector } from "wagmi/connectors/coinbaseWallet";

import {zkSyncTestnet, zkSync} from 'viem/chains'
import { WalletConnectConnector } from "wagmi/connectors/walletConnect"
import { MetaMaskConnector } from 'wagmi/connectors/metaMask'
import { InjectedConnector } from 'wagmi/connectors/injected'
import { EIP6963Connector, walletConnectProvider } from "@web3modal/wagmi";

const projectId = '630093679339d9e6a59508feafbae4ce' // TODO who owns this? make sure nethermind owns this and move to a config file may be ?

const { chains, publicClient } = configureChains([zkSyncTestnet, zkSync], [walletConnectProvider({ projectId }), publicProvider()]);

const metadata = {
  name: "zkSync remix plugin",
  description: "zkSync remix plugin",
  // url: "",
  // icons: ["https://avatars.githubusercontent.com/u/37784886"],
};

const wagmiConfig = createConfig({
  autoConnect: false,
  connectors: [
    new WalletConnectConnector({ chains, options: { projectId, showQrModal: false, metadata } }),
    new EIP6963Connector({ chains }),
    new InjectedConnector({ chains, options: { shimDisconnect: true } }),
    new CoinbaseWalletConnector({ chains, options: { appName: metadata.name } }),
  ],
  publicClient,
});

createWeb3Modal({ wagmiConfig, projectId, chains })

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)
root.render(
  <WagmiConfig config={wagmiConfig}>
     <React.StrictMode>
      <App />
     </React.StrictMode>
   </WagmiConfig>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
