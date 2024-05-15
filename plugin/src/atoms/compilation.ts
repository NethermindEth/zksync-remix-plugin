import { atom } from 'jotai'

const statusAtom = atom<string>('Compiling....')

const hashDirAtom = atom<string>('')

const isCompilingAtom = atom<boolean>(false)

type CompilationKeys =
  'status'
  | 'isCompiling'
  | 'hashDir'

interface SetCompilationValue {
  key: CompilationKeys
  value: string | boolean | string[]
}

const compilationAtom = atom(
  (get) => {
    return {
      status: get(statusAtom),
      isCompiling: get(isCompilingAtom),
      hashDir: get(hashDirAtom)
    }
  },
  (_get, set, newValue: SetCompilationValue) => {
    switch (newValue?.key) {
      case 'status':
        typeof newValue?.value === 'string' && set(statusAtom, newValue?.value)
        break
      case 'isCompiling':
        typeof newValue?.value === 'boolean' && set(isCompilingAtom, newValue?.value)
        break
      case 'hashDir':
        typeof newValue?.value === 'string' && set(hashDirAtom, newValue?.value)
        break
    }
  }
)

export {
  statusAtom,
  isCompilingAtom,
  hashDirAtom,
  compilationAtom,
  type SetCompilationValue,
  type CompilationKeys
}
