import { type DevnetAccount } from '../types/accounts'

const apiUrl = import.meta.env.VITE_API_URL ?? 'solidity-compile-remix-test.nethermind.io'
const devnetUrl = 'http://localhost:8011'
//  process.env.REACT_APP_DEVNET_URL ?? 'http://localhost:8011'
const remoteDevnetUrl = process.env.REACT_APP_REMOTE_DEVNET_URL ?? 'https://starknet-devnet-dev.nethermind.io'

interface Devnet {
  name: string
  url: string
}

const devnets: Devnet[] = [
  {
    name: 'Local Devnet',
    url: devnetUrl
  },
  {
    name: 'Remote Devnet',
    url: remoteDevnetUrl
  }
]

const getAccounts = async (
  customDevnetUrl: string = devnetUrl
): Promise<any> => {
  const initial_balance = 1000000000000*10**18;
  const private_keys = [
  '0xd293c684d884d56f8d6abd64fc76757d3664904e309a0645baf8522ab6366d9e', 
  '0xac1e735be8536c6534bb4f17f06f6afc73b2b5ba84ac2cfb12f7461b20c0bbe3',
  '0xe667e57a9b8aaa6709e51ff7d093f1c5b73b63f9987e4ab4aa9a5c699e024ee8',
  '0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110',
  '0xf12e28c0eb1ef4ff90478f6805b68d63737b7f33abfa091601140805da450d93',
  '0x850683b40d4a740aa6e745f889a6fdc8327be76e122f5aba645a5b02d0248db8',
  '0x74d8b3a188f7260f67698eb44da07397a298df5427df681ef68c45b34b61f998',
  '0xbe79721778b48bcc679b78edac0ce48306a8578186ffcb9f2ee455ae6efeace1',
  '0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c',
  '0x28a574ab2de8a00364d5dd4b07c4f2f574ef7fcc2a86a197f65abaec836d1959']
  const response:any = await fetch(`${customDevnetUrl}`,{
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({"jsonrpc": "2.0","id": "1","method": "eth_accounts","params": []})
  })
  const accounts = await response.json()
  const resp: DevnetAccount[]= await accounts.result.map((address: string, index: number) => ({
    initial_balance,
    address,
    private_key: private_keys[index]
  }));
 return resp
}

const getAccountBalance = async (
  address: string,
  customDevnetUrl: string = devnetUrl
): Promise<any> => {
  const response = await fetch(`${customDevnetUrl}`,{
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({"jsonrpc": "2.0",
    "id": "1",
    "method": "eth_getBalance",
    "params": [`${address}`, "latest"]})
  })
  const account = await response.json()
  return account.result
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
  getDevnetIndex
}

export type { Devnet, DevnetAccount }
