import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'
import App from './App'
import reportWebVitals from './reportWebVitals'
import {createWeb3Modal} from '@web3modal/wagmi/react'
import {configureChains, createConfig, WagmiConfig} from 'wagmi'
import {publicProvider} from 'wagmi/providers/public'
import {CoinbaseWalletConnector} from 'wagmi/connectors/coinbaseWallet'
import {zkSync, zkSyncSepoliaTestnet} from 'viem/chains'
import {WalletConnectConnector} from 'wagmi/connectors/walletConnect'
import {InjectedConnector} from 'wagmi/connectors/injected'
import {EIP6963Connector, walletConnectProvider} from '@web3modal/wagmi'
import {sophonTestnet, zkCandySepoliaTestnet, zkLinkNova, zkLinkNovaTestnet} from './utils/custom_chains'

const projectId: string = import.meta.env.VITE_WALLET_CONNECT_PROJECT_ID // TODO who owns this? make sure nethermind owns this

const zkSyncChains = [zkSyncSepoliaTestnet, zkSync]
const supportedChains = [zkCandySepoliaTestnet, zkLinkNova, zkLinkNovaTestnet, sophonTestnet]

const chains = [...zkSyncChains, ...supportedChains]

const {publicClient} = configureChains(
    chains,
    [walletConnectProvider({projectId}), publicProvider()]
)

const metadata = {
    name: 'zkSync remix plugin',
    description: 'zkSync remix plugin',
    url: 'https://remix.ethereum.org',
    icons: ['https://avatars.githubusercontent.com/u/37784886']
}

const wagmiConfig = createConfig({
    autoConnect: false,
    connectors: [
        new WalletConnectConnector({
            chains,
            options: {
                projectId,
                showQrModal: false,
                metadata
            }
        }),
        new EIP6963Connector({chains}),
        new InjectedConnector({
            chains,
            options: {shimDisconnect: true}
        }),
        new CoinbaseWalletConnector({
            chains,
            options: {appName: metadata.name}
        })
    ],
    publicClient
})

createWeb3Modal({
    wagmiConfig,
    projectId,
    chains
})

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)
root.render(
    <WagmiConfig config={wagmiConfig}>
        <React.StrictMode>
            <App/>
        </React.StrictMode>
    </WagmiConfig>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
