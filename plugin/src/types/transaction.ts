import { type Provider, type Wallet } from 'zksync-web3'
import { type Chain } from 'viem'

export interface Transaction {
  type: 'deploy' | 'invoke'
  txId: string
  env: 'localDevnet' | 'remoteDevnet' | 'wallet'
  account: Wallet | null
  provider: Provider | null
  chain: Chain | null
}
