// A component that reads the compiled contracts from the context and displays them in a select

import React from 'react'
import {
  getContractNameFromFullName,
  getSelectedContractIndex
} from '../../utils/utils'
import FunctionalInput from '../FunctionalInput'
import './deployedContracts.css'
import { useAtomValue, useAtom } from 'jotai'
import { deployedContractsAtom, deployedSelectedContractAtom } from '../../atoms/deployedContracts'

// eslint-disable-next-line @typescript-eslint/no-empty-interface

const DeployedContracts: React.FC = () => {
  const
    contracts = useAtomValue(deployedContractsAtom)
  const [
    selectedContract,
    setSelectedContract
  ] = useAtom(deployedSelectedContractAtom)

  function handleCompiledContractSelectionChange (event: any): void {
    event.preventDefault()
    setSelectedContract(contracts[event.target.value])
  }

  return (
    <>
      <select
        className='custom-select'
        aria-label='.form-select-sm example'
        onChange={(e) => {
          handleCompiledContractSelectionChange(e)
        }}
        defaultValue={getSelectedContractIndex(contracts, selectedContract)}
      >
        {contracts.map((contract, index) => {
          return (
            <option value={index} key={index}>
              {`${getContractNameFromFullName(contract.contractName)}, ${contract.address}`}
            </option>
          )
        })}
      </select>

      {selectedContract != null
        ? (
          <div>
            {selectedContract.abi.map(
              (abiElement, index) => {
                return abiElement.type !== 'constructor' && (
                  <div key={index} className={'methodBox'}>
                    <FunctionalInput element={abiElement}></FunctionalInput>
                  </div>
                )
              }
            )
            }
          </div>
          )
        : (
          <div>
            <p>No contract selected</p>
          </div>
          )
      }

    </>
  )
}

export default DeployedContracts
