import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import "@rainbow-me/rainbowkit/styles.css";
import { getDefaultWallets, RainbowKitProvider,darkTheme} from "@rainbow-me/rainbowkit";
import { configureChains, createConfig, WagmiConfig } from 'wagmi';
import {
  goerli
} from 'wagmi/chains';
import { alchemyProvider } from 'wagmi/providers/alchemy';
import { publicProvider } from 'wagmi/providers/public';
import { InjectedConnector } from 'wagmi/connectors/injected'

//WAGMI
const { chains, publicClient } = configureChains(
  [goerli],
  [
    publicProvider()
  ]
);

const { connectors } = getDefaultWallets({
  appName: 'Zk Plugin',
  projectId: '630093679339d9e6a59508feafbae4ce',
  chains
});

const wagmiConfig = createConfig({
  autoConnect: true,
  connectors,
  publicClient
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
