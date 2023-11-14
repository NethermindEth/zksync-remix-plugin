import { type DeployedContract } from '../types/contracts'
import { atom } from 'jotai'

const deployedContractsAtom = atom<DeployedContract[]>([])

const deployedSelectedContractAtom = atom<DeployedContract | null>(null)

export { deployedContractsAtom, deployedSelectedContractAtom }
