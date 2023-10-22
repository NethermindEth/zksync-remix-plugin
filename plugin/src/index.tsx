import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import "@rainbow-me/rainbowkit/styles.css";
import { getDefaultWallets, RainbowKitProvider,darkTheme, Chain} from "@rainbow-me/rainbowkit";
import { configureChains, createConfig, WagmiConfig } from 'wagmi';

import { publicProvider } from 'wagmi/providers/public';
import { InjectedConnector } from 'wagmi/connectors/injected'

const zksync: Chain = {
  id: 280,
  name: 'zkSync Era Testnet',
  network: 'zkSync Era Testnet',
  iconUrl: 'https://example.com/icon.svg',
  iconBackground: '#fff',
  nativeCurrency: {
    decimals: 18,
    name: 'Ethereum',
    symbol: 'ETH',
  },
  rpcUrls: {
    public: { http: ['https://testnet.era.zksync.dev'] },
    default: { http: ['https://testnet.era.zksync.dev'] },
  },
  blockExplorers: {
    default: { name: 'zkSync Era Testnet Explorer', url: 'https://goerli.explorer.zksync.io/' },
    etherscan: { name: 'zkSync Era Testnet Explorer', url: 'https://goerli.explorer.zksync.io/' },
  },
};

//WAGMI
const { chains, publicClient } = configureChains(
  [zksync],
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
     {/* <React.StrictMode> */}
      <App />
     {/* </React.StrictMode> */}
    </RainbowKitProvider>
  </WagmiConfig>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
