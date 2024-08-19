const devnetUrl = 'http://127.0.0.1:8011'

type Network = 'goerli-alpha' | 'goerli-alpha-2' | 'mainnet-alpha'

const networks = [
  {
    name: 'Testnet',
    value: 'goerli-alpha'
  },
  {
    name: 'Testnet 2',
    value: 'goerli-alpha-2'
  },
  {
    name: 'Mainnet',
    value: 'mainnet-alpha'
  }
]

const networkExplorerUrls = {
  voyager: {
    'goerli-alpha': 'https://goerli.voyager.online',
    'goerli-alpha-2': 'https://goerli-2.voyager.online',
    'mainnet-alpha': 'https://voyager.online'
  },
  starkscan: {
    'goerli-alpha': 'https://testnet.starkscan.co',
    'goerli-alpha-2': 'https://testnet-2.starkscan.co',
    'mainnet-alpha': 'https://starkscan.co'
  }
}

const licenses = [
  'No License (None)',
  'The Unlicense (Unlicense)',
  'MIT License (MIT)',
  'GNU General Public License v2.0 (GNU GPLv2)',
  'GNU General Public License v3.0 (GNU GPLv3)',
  'GNU Lesser General Public License v2.1 (GNU LGPLv2.1)',
  'GNU Lesser General Public License v3.0 (GNU LGPLv3)',
  'BSD 2-clause "Simplified" license (BSD-2-Clause)',
  'BSD 3-clause "New" Or "Revisited license (BSD-3-Clause)',
  'Mozilla Public License 2.0 (MPL-2.0)',
  'Open Software License 3.0 (OSL-3.0)',
  'Apache 2.0 (Apache-2.0)',
  'GNU Affero General Public License (GNU AGPLv3)',
  'Business Source License (BSL 1.1)'
]

export const FILES_NOT_IN_CONTRACTS_MESSAGE =
  'Warning: Only files within the /contracts/* folder structure are included in the project compilation. The following smart contracts defined outside this structure will not be available for deployment:'

export const PLUGIN_INFO_CONTENT_ARRAY = [
    'The zkSync Remix Plugin is in Alpha',
    'Solidity contracts are compiled on a server hosted by Nethermind'
  ]

export { devnetUrl, networks, networkExplorerUrls, licenses }

export type { Network }
