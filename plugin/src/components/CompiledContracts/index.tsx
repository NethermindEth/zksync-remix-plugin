// A component that reads the compiled contracts from the context and displays them in a select

import React from 'react'
import {
  getContractNameFromFullName,
  getSelectedContractIndex
} from '../../utils/utils'
import { contractsAtom, selectedContractAtom } from '../../atoms/compiledContracts'
import { useAtomValue } from 'jotai/react/useAtomValue'
import { useAtom } from 'jotai/react/useAtom'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  show: 'class' | 'contract'
}

const CompiledContracts: React.FC<CompiledContractsProps> = (props) => {
  const contracts = useAtomValue(contractsAtom)
  const [selectedContract, setSelectedContract] = useAtom(selectedContractAtom)

  function handleCompiledContractSelectionChange (event: any): void {
    event.preventDefault()
    setSelectedContract(contracts[event.target.value])
  }

  return (
    <select
      className="custom-select"
      aria-label=".form-select-sm example"
      onChange={(e) => {
        handleCompiledContractSelectionChange(e)
      }}
      defaultValue={getSelectedContractIndex(contracts, selectedContract)}
    >
      {contracts.map((contract, index) => {
        return (
          <option value={index} key={index}>
            {`${getContractNameFromFullName(contract.contractName)}`}
          </option>
        )
      })}
    </select>
  )
}

export default CompiledContracts
