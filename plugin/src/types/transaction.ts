import { type Provider, type Account, type AccountInterface, type ProviderInterface } from 'starknet'

export interface Transaction {
  type: 'deploy' | 'invoke'
  txId: string
  env: string
  account: Account | AccountInterface | null
  provider: Provider | ProviderInterface | null
}
