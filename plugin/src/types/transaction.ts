import { Provider, Wallet } from 'zksync-web3'

export interface Transaction {
  type: 'deploy' | 'invoke'
  txId: string
  env: string
  account: Wallet | null
  provider: Provider | null
}
