import { atom } from 'jotai'

const statusAtom = atom<string>('Verifying...')

const currentFilenameAtom = atom<string>('')

const isVerifyingAtom = atom<boolean>(false)

const isValidSolidityAtom = atom<boolean>(false)

const noFileSelectedAtom = atom<boolean>(false)

const hashDirAtom = atom<string>('')

const tomlPathsAtom = atom<string[]>([])

const activeTomlPathAtom = atom<string>('')

type VerificationKeys =
  'status'
  | 'currentFilename'
  | 'isVerifying'
  | 'isValidSolidity'
  | 'noFileSelected'
  | 'hashDir'
  | 'tomlPaths'
  | 'activeTomlPath'

interface SetVerificationValue {
  key: VerificationKeys
  value: string | boolean | string[]
}

const verificationAtom = atom(
  (get) => {
    return {
      status: get(statusAtom),
      currentFilename: get(currentFilenameAtom),
      isVerifying: get(isVerifyingAtom),
      isValidSolidity: get(isValidSolidityAtom),
      noFileSelected: get(noFileSelectedAtom),
      hashDir: get(hashDirAtom),
      tomlPaths: get(tomlPathsAtom),
      activeTomlPath: get(activeTomlPathAtom)
    }
  },
  (_get, set, newValue: SetVerificationValue) => {
    switch (newValue?.key) {
      case 'status':
        typeof newValue?.value === 'string' && set(statusAtom, newValue?.value)
        break
      case 'currentFilename':
        typeof newValue?.value === 'string' && set(currentFilenameAtom, newValue?.value)
        break
      case 'isVerifying':
        typeof newValue?.value === 'boolean' && set(isVerifyingAtom, newValue?.value)
        break
      case 'isValidSolidity':
        typeof newValue?.value === 'boolean' && set(isValidSolidityAtom, newValue?.value)
        break
      case 'noFileSelected':
        typeof newValue?.value === 'boolean' && set(noFileSelectedAtom, newValue?.value)
        break
      case 'hashDir':
        typeof newValue?.value === 'string' && set(hashDirAtom, newValue?.value)
        break
      case 'tomlPaths':
        Array.isArray(newValue?.value) && set(tomlPathsAtom, newValue?.value)
        break
      case 'activeTomlPath':
        typeof newValue?.value === 'string' && set(activeTomlPathAtom, newValue?.value)
        break
    }
  }
)

export {
  statusAtom,
  currentFilenameAtom,
  isVerifyingAtom,
  isValidSolidityAtom,
  noFileSelectedAtom,
  hashDirAtom,
  tomlPathsAtom,
  activeTomlPathAtom,
  verificationAtom,
  type SetVerificationValue,
  type VerificationKeys
}
