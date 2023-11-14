import { atom } from 'jotai'
import { type Input } from '../types/contracts'

const isDeployingAtom = atom<boolean>(false)

const deployStatusAtom = atom<string>('')

const constructorInputsAtom = atom<Input[]>([])

const notEnoughInputsAtom = atom<boolean>(false)

type Key = 'isDeploying' | 'deployStatus' | 'constructorInputs' | 'notEnoughInputs'

interface SetDeploymentAtom {
  key: Key
  value: boolean | string | Input[]
}

const deploymentAtom = atom((get) => {
  return {
    isDeploying: get(isDeployingAtom),
    deployStatus: get(deployStatusAtom),
    constructorInputs: get(constructorInputsAtom),
    notEnoughInputs: get(notEnoughInputsAtom)
  }
}, (_get, set, newValue: SetDeploymentAtom) => {
  switch (newValue?.key) {
    case 'isDeploying': typeof newValue?.value === 'boolean' && set(isDeployingAtom, newValue?.value); break
    case 'deployStatus': typeof newValue?.value === 'string' && set(deployStatusAtom, newValue?.value); break
    case 'constructorInputs': Array.isArray(newValue?.value) && set(constructorInputsAtom, newValue?.value); break
    case 'notEnoughInputs': typeof newValue?.value === 'boolean' && set(notEnoughInputsAtom, newValue?.value); break
  }
}
)

export { isDeployingAtom, deployStatusAtom, constructorInputsAtom, notEnoughInputsAtom, deploymentAtom }
