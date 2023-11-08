import { createContext } from 'react'
import {
  type Provider,
  type Wallet
} from 'zksync-web3'

const ConnectionContext = createContext({
  provider: null as Provider | null,
  setProvider: (_: Provider | null) => {},
  account: null as Wallet | null,
  setAccount: (_: Wallet | null) => {}
})

export { ConnectionContext }
