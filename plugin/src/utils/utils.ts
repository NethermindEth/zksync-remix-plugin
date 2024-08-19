import { ContractInputType } from '@/components/ConstructorInput'
import { type DevnetAccount } from '../types/accounts'
import { type Abi, type AbiElement, type Contract, type Input } from '../types/contracts'
import { type Network, networkExplorerUrls } from './constants'

const getFileExtension = (filename: string): string => filename.split('.').pop() ?? ''

const getFileNameFromPath = (path: string): string => path.split('/').pop() ?? ''

const getContractNameFromFullName = (fullName: string): string => fullName.split('.')[0]

const artifactFolder = (path: string): string => {
  return path.concat('/artifacts')
}

const artifactFilename = (ext: '.json' | '.casm', filename: string): string => filename.split('.')[0].concat(ext)

const getContractByClassHash = (classHash: string, contracts: Contract[]): Contract | undefined => {
  return contracts.find((contract) => contract.sourceName === classHash)
}

const getShortenedHash = (address: string, first: number, second: number): string => {
  return `${address.slice(0, first)}...${address.slice(-1 * second)}`
}

const getConstructor = (abi: Abi): AbiElement | undefined => {
  return abi.find((item) => item.name === 'constructor')
}

const getContractFunctions = (abi: Abi): AbiElement[] => {
  const contractFunctions = abi.filter((item) => item.type === 'function' && item.name !== 'constructor')
  return contractFunctions
}

function generateInputName(input: Input): string {
  return `${input.name} (${input.type})`
}

const getParameterType = (parameter: string): string | undefined => {
  const type = parameter.split('::').pop()
  if (type === 'u256') return 'u256 (low, high)'
  return type
}

const getSelectedContractIndex = (contracts: Contract[], selectedContract: Contract | null): number => {
  if (selectedContract != null) {
    return contracts.findIndex(
      (contract) =>
        `${contract.sourceName}:${contract.contractName}` ===
        `${selectedContract.sourceName}:${selectedContract.contractName}`
    )
  }
  return 0
}

const getSelectedAccountIndex = (accounts: DevnetAccount[], selectedAccount: DevnetAccount | null): number => {
  if (selectedAccount != null) {
    return accounts.findIndex((account) => account.address === selectedAccount.address)
  }
  return -1
}

const getRoundedNumber = (number: number, decimals: number): number => {
  return Math.round(number * Math.pow(10, decimals)) / Math.pow(10, decimals)
}

const weiToEth = (wei: number): number => {
  return wei / 10 ** 18
}

const getExplorerUrl = (explorer: keyof typeof networkExplorerUrls, chain: Network): string =>
  networkExplorerUrls[explorer][chain]

const trimStr = (str?: string, strip?: number): string => {
  if (str == null) {
    return ''
  }
  const length = str.length
  return `${str?.slice(0, strip ?? 6)}...${str?.slice(length - (strip ?? 6))}`
}

const isSolidityArrayType = (str: string) => {
  // Regular expression to match Solidity array types
  const arrayTypeRegex = /^(uint|int|bytes|string|address|bool)(\d{1,3})?(\[\])+$/

  if (str === 'bytes[]' || str === 'string[]') {
    return true
  }

  if (arrayTypeRegex.test(str)) {
    const match = str.match(arrayTypeRegex)
    if (match && (match[1] === 'uint' || match[1] === 'int')) {
      const bitSize = parseInt(match[2] || '256')
      return bitSize >= 8 && bitSize <= 256 && bitSize % 8 === 0
    }

    if (match && match[1] === 'bytes' && match[2]) {
      const byteSize = parseInt(match[2])
      return byteSize >= 1 && byteSize <= 32
    }
    return true
  }
  return false
}

const parseContractInputs = (inputs: ContractInputType) =>
  inputs.map((input) => {
    if (isSolidityArrayType(input.internalType)) {
      try {
        const parsedValue = JSON.parse(input.value)
        return parsedValue
      } catch (error) {
        console.error(`Failed to parse constructor input ${error}`)
        return input.value
      }
    }
    return input.value
  })

export {
  getFileExtension,
  getFileNameFromPath,
  getContractNameFromFullName,
  artifactFolder,
  artifactFilename,
  getContractByClassHash,
  getShortenedHash,
  getConstructor,
  getContractFunctions,
  getParameterType,
  getSelectedContractIndex,
  getSelectedAccountIndex,
  getRoundedNumber,
  weiToEth,
  getExplorerUrl,
  generateInputName,
  trimStr,
  parseContractInputs
}
