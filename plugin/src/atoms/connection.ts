import { atom } from 'jotai'
import { type Provider, type Signer, type Wallet, type Web3Provider } from 'zksync-ethers'

const providerAtom = atom<Provider | Web3Provider | null>(null)

const accountAtom = atom<Wallet | Signer | null>(null)

export { providerAtom, accountAtom }
