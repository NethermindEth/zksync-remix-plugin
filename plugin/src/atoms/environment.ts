import { atom } from 'jotai'
import { type Devnet, type DevnetAccount, devnets } from '../utils/network'

const devnetAtom = atom<Devnet>(devnets[1])

const envAtom = atom<string>('remoteDevnet')

const isDevnetAliveAtom = atom<boolean>(true)

const selectedDevnetAccountAtom = atom<null | DevnetAccount>(null)

const availableDevnetAccountsAtom = atom<DevnetAccount[]>([])

export {
  devnetAtom,
  envAtom,
  isDevnetAliveAtom,
  selectedDevnetAccountAtom,
  availableDevnetAccountsAtom
}
