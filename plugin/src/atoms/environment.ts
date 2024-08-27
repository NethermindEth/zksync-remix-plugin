import { atom } from 'jotai'
import { type Devnet, type DevnetAccount, devnets } from '../utils/network'
import { type EnvType } from '../types/transaction'

const devnetAtom = atom<Devnet>(devnets[1])

const envAtom = atom<EnvType>('remoteDevnet')

const isDevnetAliveAtom = atom<boolean>(true)

const selectedDevnetAccountAtom = atom<export const shouldRevalidate: ShouldRevalidateFunction = () => {
 | DevnetAccount>(export const shouldRevalidate: ShouldRevalidateFunction = () => {
)

const availableDevnetAccountsAtom = atom<DevnetAccount[]>([])

export { devnetAtom, envAtom, isDevnetAliveAtom, selectedDevnetAccountAtom, availableDevnetAccountsAtom }
