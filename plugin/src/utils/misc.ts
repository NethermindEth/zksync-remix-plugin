// Set of functions I thought I might need to use in the future.
// TODO: erase if not neeeded.

import { devnetUrl } from './constants'
import { Provider } from 'zksync-web3'

const getProvider = (network: string) => {
  switch (network) {
    case 'mainnet-alpha':
      return new Provider('https://sequencer.zksync.io')
    case 'goerli-alpha':
      return new Provider('https://goerli.zksync.io')
    case 'goerli-alpha-2':
      return new Provider('https://goerli.zksync.io')
    case devnetUrl:
      return new Provider(devnetUrl)
    default:
      return new Provider('https://goerli.zksync.io')
  }
}

interface IExplorerSelector {
  path?: string
  text?: string
  title?: string
  isInline?: boolean
  isNetworkVisible?: boolean
  isTextVisible?: boolean
  controlHook: IUseCurrentExplorer
}

interface IUseCurrentExplorer {
  explorer: 'voyager' | 'starkscan'
  setExplorer: React.Dispatch<React.SetStateAction<'voyager' | 'starkscan'>>
}

export { getProvider, type IExplorerSelector, type IUseCurrentExplorer }
