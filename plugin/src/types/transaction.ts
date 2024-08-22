import { type Provider, type Signer, type Wallet } from 'zksync-ethers'
import { type Chain, type ChainFormatters } from 'viem'

export type EnvType = 'localDevnet' | 'remoteDevnet' | 'wallet' | 'manual' | 'customNetwork'

export interface Transaction {
  type: 'deploy' | 'invoke'
  txId: string
  env: EnvType
  account: Wallet | Signer | null
  provider: Provider | null
  chain: Chain | undefined | null
  value: string | undefined | null
}

export const mockManualChain: Chain<ChainFormatters> = {
  id: 0,
  nativeCurrency: {
    name: 'ETH',
    symbol: 'ETH',
    decimals: 18
  },
  rpcUrls: {
    default: { http: [''], webSocket: [''] },
    public: { http: [''], webSocket: [''] }
  },
  network: 'testnet',
  name: 'testnet',
  blockExplorers: {
    default: {
      name: 'testnet',
      url: 'https://sepolia.explorer.zksync.io/'
    }
  }
}
