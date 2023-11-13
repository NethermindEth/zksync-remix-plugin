import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import "@rainbow-me/rainbowkit/styles.css";
import { getDefaultWallets, RainbowKitProvider, darkTheme, Chain} from "@rainbow-me/rainbowkit";

import { configureChains, createConfig, WagmiConfig } from 'wagmi';
import { publicProvider } from 'wagmi/providers/public';
import {zkSyncTestnet} from 'viem/chains'

const { chains, publicClient } = configureChains(
  [zkSyncTestnet],
  [publicProvider()]
);


const { connectors } = getDefaultWallets({
  appName: 'Zk',
  projectId: '630093679339d9e6a59508feafbae4ce', // only for walletconnect wallet, TODO is there a better place to store this id?
  chains
});

const wagmiConfig = createConfig({
  autoConnect: false,
  connectors,
  publicClient,
})

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)
root.render(
  <WagmiConfig config={wagmiConfig}>
    <RainbowKitProvider chains={chains} theme={darkTheme()}>
     <React.StrictMode>
      <App />
     </React.StrictMode> 
     </RainbowKitProvider>
   </WagmiConfig>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
