import { type Transaction } from '../types/transaction'
import { atom } from 'jotai'

const transactionsAtom = atom<Transaction[]>([])

export { transactionsAtom }
