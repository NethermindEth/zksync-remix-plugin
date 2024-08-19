import { atom } from 'jotai'

export type CompilationType = 'PROJECT' | 'SINGLE_FILE' | 'NONE'
const compileStatusAtom = atom<string>('Compiling....')
const hashDirAtom = atom<string>('')
const isCompilingAtom = atom<boolean>(false)
const compilationTypeAtom = atom<CompilationType>('NONE')
type CompilationKeys = 'status' | 'isCompiling' | 'hashDir' | 'errorMessages' | 'compilationType'
const compileErrorMessagesAtom = atom<string[]>([])

interface SetCompilationValue {
  key: CompilationKeys
  value: string | boolean | string[]
}

const compilationAtom = atom(
  (get) => {
    return {
      status: get(compileStatusAtom),
      isCompiling: get(isCompilingAtom),
      hashDir: get(hashDirAtom),
      errorMessages: get(compileErrorMessagesAtom),
      compilationType: get(compilationTypeAtom)
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
      case 'errorMessages':
        Array.isArray(newValue?.value) && set(compileErrorMessagesAtom, newValue?.value)
        break
      case 'compilationType':
        typeof newValue?.value === 'string' && set(compilationTypeAtom, newValue?.value as CompilationType)
        break
    }
  }
)

export {
  compileStatusAtom,
  isCompilingAtom,
  hashDirAtom,
  compilationAtom,
  compileErrorMessagesAtom,
  compilationTypeAtom,
  type SetCompilationValue,
  type CompilationKeys
}
