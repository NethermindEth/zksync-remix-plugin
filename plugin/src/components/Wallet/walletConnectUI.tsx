import {Web3Button, Web3Modal} from '@web3modal/react'
import {WagmiConfig} from 'wagmi'
import {PROJECT_ID} from '../../services/constant'

export function WalletConnectUI({ethereumClient, wagmiConfig, theme}) {
  console.log('wagmiConfig', wagmiConfig)
  return (
    <div>
      <div style={{display: 'inline-block'}}>
        <WagmiConfig config={wagmiConfig}>
          <Web3Button label="Connect to a wallet" />
        </WagmiConfig>
      </div>
      <Web3Modal projectId='6e2396d2519b270b5d8d35e80dc66ec9' ethereumClient={ethereumClient} themeMode={theme} />
    </div>
  )
}
