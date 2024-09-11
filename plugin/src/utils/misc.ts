import { EnvType } from '@/types/transaction'

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

export const envName = (env: EnvType): string => {
  switch (env) {
    case 'remoteDevnet':
      return 'Remote Devnet'
    case 'wallet':
      return 'Wallet'
    case 'manual':
      return 'Manual'
    case 'localDevnet':
      return 'Local Devnet'
    case 'customNetwork':
      return 'Custom Network'
    default:
      return 'Unknown'
  }
}

export { type IExplorerSelector, type IUseCurrentExplorer }
