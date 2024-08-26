import { atom } from 'jotai'
import { type Devnet, type DevnetAccount, devnets } from '../utils/network'
import { type EnvType } from '../types/transaction'

const devnetAtom = atom<Devnet>(devnets[1])

const envAtom = atom<EnvType>('remoteDevnet')

const isDevnetAliveAtom = atom<boolean>(true)

const selectedDevnetAccountAtom = atom<null | DevnetAccount>(null)

const availableDevnetAccountsAtom = atom<DevnetAccount[]>([])
const customNetworkAtom = atom('')
const isCustomNetworkAliveAtom = atom<boolean>(false)

export {
  devnetAtom,
  envAtom,
  isDevnetAliveAtom,
  selectedDevnetAccountAtom,
  availableDevnetAccountsAtom,
  customNetworkAtom,
  isCustomNetworkAliveAtom
}
