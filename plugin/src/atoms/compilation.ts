import { atom } from 'jotai'

const compileStatusAtom = atom<string>('Compiling....')
const hashDirAtom = atom<string>('')
const isCompilingAtom = atom<boolean>(false)
type CompilationKeys = 'status' | 'isCompiling' | 'hashDir'

interface SetCompilationValue {
  key: CompilationKeys
  value: string | boolean | string[]
}

const compilationAtom = atom(
  (get) => {
    return {
      status: get(compileStatusAtom),
      isCompiling: get(isCompilingAtom),
      hashDir: get(hashDirAtom)
    }
  },
  (_get, set, newValue: SetCompilationValue) => {
    switch (newValue?.key) {
      case 'status':
        typeof newValue?.value === 'string' && set(compileStatusAtom, newValue?.value)
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
  compileStatusAtom,
  isCompilingAtom,
  hashDirAtom,
  compilationAtom,
  type SetCompilationValue,
  type CompilationKeys
}
