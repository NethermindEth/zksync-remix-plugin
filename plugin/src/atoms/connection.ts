import { atom } from 'jotai'
import { type Provider, type Wallet } from 'zksync-web3'

const providerAtom = atom<Provider | null>(null)

const accountAtom = atom<Wallet | null>(null)

export { providerAtom, accountAtom }
