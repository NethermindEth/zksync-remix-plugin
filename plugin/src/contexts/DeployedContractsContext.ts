import { createContext } from 'react'
import { type DeployedContract } from '../types/contracts'

const DeployedContractsContext = createContext({
  contracts: [] as DeployedContract[],
  setContracts: (contracts: DeployedContract[]) => {
  },
  selectedContract: null as DeployedContract | null,
  setSelectedContract: (contract: DeployedContract | null) => {
  }
})

export { DeployedContractsContext }
