import { atom } from 'jotai'

const solidityVersionAtom = atom<string>('latest')

const versionsAtom = atom<string[]>([])

export { solidityVersionAtom, versionsAtom }
