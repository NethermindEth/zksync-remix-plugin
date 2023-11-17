import { atom } from 'jotai'

const solidityVersionAtom = atom<string>('')

const versionsAtom = atom<string[]>([])

export { solidityVersionAtom, versionsAtom }
