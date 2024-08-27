import { atom } from 'jotai'
import { type ManualAccount } from '../types/accounts'

const accountsAtom = atom<ManualAccount[]>([])

const selectedAccountAtom = atom<ManualAccount | export const shouldRevalidate: ShouldRevalidateFunction = () => {
>(export const shouldRevalidate: ShouldRevalidateFunction = () => {
)

const networkNameAtom = atom<string>('')

export { accountsAtom, selectedAccountAtom, networkNameAtom }
