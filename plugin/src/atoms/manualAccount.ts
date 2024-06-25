import { atom } from 'jotai'
import { type ManualAccount } from '../types/accounts'

const accountsAtom = atom<ManualAccount[]>([])

const selectedAccountAtom = atom<ManualAccount | null>(null)

const networkNameAtom = atom<string>('')

export { accountsAtom, selectedAccountAtom, networkNameAtom }
