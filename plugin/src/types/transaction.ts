import { type Provider, type Signer, type Wallet } from 'zksync-web3'
import { type Chain } from 'viem'

export type EnvType = 'localDevnet' | 'remoteDevnet' | 'wallet' | 'manual'

export interface Transaction {
  type: 'deploy' | 'invoke'
  txId: string
  env: EnvType
  account: Wallet | Signer | null
  provider: Provider | null
  chain: Chain | undefined | null
}
