import { atom } from 'jotai'

const verificatationStatusAtom = atom<string>('Verifying...')

const isVerifyingAtom = atom<boolean>(false)

type VerificationKeys = 'status' | 'isVerifying'

interface SetVerificationValue {
  key: VerificationKeys
  value: string | boolean | string[]
}

const verificationAtom = atom(
  (get) => {
    return {
      status: get(verificatationStatusAtom),
      isVerifying: get(isVerifyingAtom)
    }
  },
  (_get, set, newValue: SetVerificationValue) => {
    switch (newValue?.key) {
      case 'status':
        typeof newValue?.value === 'string' && set(verificatationStatusAtom, newValue?.value)
        break
      case 'isVerifying':
        typeof newValue?.value === 'boolean' && set(isVerifyingAtom, newValue?.value)
        break
    }
  }
)

export { verificatationStatusAtom, isVerifyingAtom, verificationAtom, type SetVerificationValue, type VerificationKeys }
