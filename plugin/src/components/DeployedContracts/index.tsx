// A component that reads the compiled contracts from the context and displays them in a select

import React, { useContext } from 'react'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import {
  getContractNameFromFullName,
  getSelectedContractIndex,
  getShortenedHash
} from '../../utils/utils'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'
import Container from "../../ui_components/Container";
import FunctionalInput from "../FunctionalInput";
import './deployedContracts.css'

// eslint-disable-next-line @typescript-eslint/no-empty-interface

const DeployedContracts: React.FC = () => {
  const { contracts, selectedContract, setSelectedContract } = useContext(
    DeployedContractsContext
  )
  function handleCompiledContractSelectionChange (event: any): void {
    event.preventDefault()
    setSelectedContract(contracts[event.target.value])
  }

  return (
    <>
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
              {`${getContractNameFromFullName(contract.contractName)}, ${contract.address}`}
            </option>
          )
        })}
      </select>

      {selectedContract != null ? (
          <div>
              { selectedContract.abi.map(
                    (abiElement, index) => {
                        return abiElement.type !== 'constructor' && (
                            <div key={index} className={"methodBox"}>
                                <FunctionalInput element={abiElement}></FunctionalInput>
                            </div>
                        )
                    }
                )
              }
          </div>
          ) : (
            <div>
              <p>No contract selected</p>
            </div>
          )
      }

    </>
  )
}

export default DeployedContracts
