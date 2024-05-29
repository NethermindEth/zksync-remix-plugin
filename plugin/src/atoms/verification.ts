import { atom } from 'jotai'

const statusAtom = atom<string>('Verifying...')

const isVerifyingAtom = atom<boolean>(false)

type VerificationKeys = 'status' | 'isVerifying'

interface SetVerificationValue {
  key: VerificationKeys
  value: string | boolean | string[]
}

const verificationAtom = atom(
  (get) => {
    return {
      status: get(statusAtom),
      isVerifying: get(isVerifyingAtom)
    }
  },
  (_get, set, newValue: SetVerificationValue) => {
    switch (newValue?.key) {
      case 'status':
        typeof newValue?.value === 'string' && set(statusAtom, newValue?.value)
        break
      case 'isVerifying':
        typeof newValue?.value === 'boolean' && set(isVerifyingAtom, newValue?.value)
        break
    }
  }
)

export { statusAtom, isVerifyingAtom, verificationAtom, type SetVerificationValue, type VerificationKeys }
