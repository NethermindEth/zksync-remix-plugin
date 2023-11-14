import { atom } from 'jotai'
import { Provider, Wallet } from 'zksync-web3'

const providerAtom = atom<Provider | null>(null)

const accountAtom = atom<Wallet | null>(null)

export { providerAtom, accountAtom }
