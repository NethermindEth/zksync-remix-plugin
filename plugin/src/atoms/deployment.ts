import { atom } from 'jotai'
import { type Input } from '../types/contracts'

type DeploymentStatus = 'IDLE' | 'IN_PROGRESS' | 'ERROR' | 'DONE'

const deployStatusAtom = atom<DeploymentStatus>('IDLE')

const constructorInputsAtom = atom<Input[]>([])

const notEnoughInputsAtom = atom<boolean>(false)

type Key = 'deployStatus' | 'constructorInputs' | 'notEnoughInputs'

interface SetDeploymentAtom {
  key: Key
  value: boolean | string | Input[]
}

const deploymentAtom = atom(
  (get) => {
    return {
      deployStatus: get(deployStatusAtom),
      constructorInputs: get(constructorInputsAtom),
      notEnoughInputs: get(notEnoughInputsAtom)
    }
  },
  (_get, set, newValue: SetDeploymentAtom) => {
    switch (newValue?.key) {
      case 'deployStatus':
        typeof newValue?.value === 'string' && set(deployStatusAtom, newValue?.value as DeploymentStatus)
        break
      case 'constructorInputs':
        Array.isArray(newValue?.value) && set(constructorInputsAtom, newValue?.value)
        break
      case 'notEnoughInputs':
        typeof newValue?.value === 'boolean' && set(notEnoughInputsAtom, newValue?.value)
        break
    }
  }
)

export { deployStatusAtom, constructorInputsAtom, notEnoughInputsAtom, deploymentAtom }
