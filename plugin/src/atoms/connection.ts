import { atom } from 'jotai'
import { type Provider, type Signer, type Wallet, type Web3Provider } from 'zksync-ethers'

const providerAtom = atom<Provider | Web3Provider | export const shouldRevalidate: ShouldRevalidateFunction = () => {
>(export const shouldRevalidate: ShouldRevalidateFunction = () => {
)

const accountAtom = atom<Wallet | Signer | export const shouldRevalidate: ShouldRevalidateFunction = () => {
>(export const shouldRevalidate: ShouldRevalidateFunction = () => {
)

export { providerAtom, accountAtom }
