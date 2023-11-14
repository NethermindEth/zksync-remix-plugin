import { atom } from 'jotai'
import { type Contract } from '../types/contracts'

const contractsAtom = atom<Contract[]>([])

const selectedContractAtom = atom<Contract | null>(null)

export { contractsAtom, selectedContractAtom }
