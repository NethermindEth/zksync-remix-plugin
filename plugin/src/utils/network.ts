import { type DevnetAccount } from '../types/accounts'
import { Wallet } from 'zksync-ethers'

const apiUrl: string = import.meta.env.VITE_API_URL ?? 'solidity-compile-remix-test.nethermind.io'
const devnetUrl = import.meta.env.VITE_DEVNET_URL ?? 'http://localhost:8011'
const remoteDevnetUrl = process.env.VITE_REMOTE_DEVNET_URL ?? 'https://zksync-devnet.nethermind.io'

interface Devnet {
  name: string
  url: string
  id: string
}

const devnets: Devnet[] = [
  {
    id: 'localDevnet',
    name: 'Local Devnet',
    url: devnetUrl
  },
  {
    id: 'remoteDevnet',
    name: 'Remote Devnet',
    url: remoteDevnetUrl
  }
]

const testnetUrl = 'https://testnet.era.zksync.dev'

const getAccounts = async (customDevnetUrl: string): Promise<DevnetAccount[]> => {
  const privateKeys = [
    '0xac1e735be8536c6534bb4f17f06f6afc73b2b5ba84ac2cfb12f7461b20c0bbe3',
    '0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c',
    '0x28a574ab2de8a00364d5dd4b07c4f2f574ef7fcc2a86a197f65abaec836d1959',
    '0xe667e57a9b8aaa6709e51ff7d093f1c5b73b63f9987e4ab4aa9a5c699e024ee8',
    '0xd293c684d884d56f8d6abd64fc76757d3664904e309a0645baf8522ab6366d9e',
    '0xf12e28c0eb1ef4ff90478f6805b68d63737b7f33abfa091601140805da450d93',
    '0x74d8b3a188f7260f67698eb44da07397a298df5427df681ef68c45b34b61f998',
    '0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110',
    '0xbe79721778b48bcc679b78edac0ce48306a8578186ffcb9f2ee455ae6efeace1',
    '0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8'
  ]

  const accountPromises: Array<Promise<DevnetAccount>> = privateKeys.map(async (privateKey: string) => {
    const wallet = new Wallet(privateKey)
    const address = wallet.address

    try {
      const initialBalance = await getAccountBalance(address, customDevnetUrl)

      const result: DevnetAccount = {
        initial_balance: initialBalance,
        address,
        private_key: privateKey
      }

      return result
    } catch (error) {
      console.error(`Failed to get balance for address ${address}: `, error)

      const result: DevnetAccount = {
        initial_balance: 0,
        address,
        private_key: privateKey
      }

      return result
    }
  })

  return await Promise.all(accountPromises)
}

const updateBalances = async (accounts: DevnetAccount[], customDevnetUrl: string): Promise<DevnetAccount[]> => {
  const accountPromises: Array<Promise<DevnetAccount>> = accounts.map(async (account: DevnetAccount) => {
    try {
      const initialBalance = await getAccountBalance(account.address, customDevnetUrl)

      const result: DevnetAccount = {
        initial_balance: initialBalance,
        address: account.address,
        private_key: account.private_key
      }

      return result
    } catch (error) {
      console.error(`Failed to get balance for address ${account.address}: `, error)

      const result: DevnetAccount = {
        initial_balance: 0,
        address: account.address,
        private_key: account.private_key
      }

      return result
    }
  })

  return await Promise.all(accountPromises)
}

const getAccountBalance = async (address: string, customDevnetUrl: string): Promise<number> => {
  try {
    const response = await fetch(`${customDevnetUrl}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: '1',
        method: 'eth_getBalance',
        params: [`${address}`, 'latest']
      })
    })
    const account = await response.json()
    const numberHex = account.result
    return parseInt(numberHex, 16)
  } catch (error) {
    console.warn(`Failed to fetch account balance for ${address} from ${customDevnetUrl} endpoint`)
    return 0
  }
}

const getDevnetUrl = (network: string): string => {
  const devnet = devnets.find((devnet) => devnet.name === network)
  if (devnet == null) throw new Error('Devnet not found')
  return devnet.url
}

const getDevnetName = (url: string): string => {
  const devnet = devnets.find((devnet) => devnet.url === url)
  if (devnet == null) throw new Error('Devnet not found')
  return devnet.name
}

const getDevnetIndex = (devnets: Devnet[], devnet: Devnet): number => {
  return devnets.findIndex((item) => item.name === devnet.name)
}

export {
  apiUrl,
  devnetUrl,
  devnets,
  getAccounts,
  getAccountBalance,
  getDevnetUrl,
  getDevnetName,
  getDevnetIndex,
  updateBalances,
  testnetUrl
}

export type { Devnet, DevnetAccount }
export const ZKSYNC_SEPOLIA_RPC_URL = process.env.VITE_ZKSYNC_SEPOLIA_RPC_URL || 'https://sepolia.era.zksync.dev'
export const ZKSYNC_SEPOLIA_FAUCET_URL =
  process.env.VITE_ZKSYNC_SEPOLIA_FAUCET_URL || 'https://learnweb3.io/faucets/zksync_sepolia/'
