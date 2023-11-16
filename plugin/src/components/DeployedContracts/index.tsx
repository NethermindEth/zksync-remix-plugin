// A component that reads the compiled contracts from the context and displays them in a select

import React from 'react'
import { getContractNameFromFullName, getShortenedHash } from '../../utils/utils'
import FunctionalInput from '../FunctionalInput'
import './deployedContracts.css'
import { useAtom, useAtomValue } from 'jotai'
import { deployedContractsAtom, deployedSelectedContractAtom } from '../../atoms/deployedContracts'
import * as D from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'

// eslint-disable-next-line @typescript-eslint/no-empty-interface

const DeployedContracts: React.FC = () => {
  const
    contracts = useAtomValue(deployedContractsAtom)
  const [
    selectedContract,
    setSelectedContract
  ] = useAtom(deployedSelectedContractAtom)

  const [dropdownControl, setDropdownControl] = React.useState(false)

  return (
    <>
      <D.Root open={dropdownControl} onOpenChange={(e) => { setDropdownControl(e) }}>
        <D.Trigger>
          <div className="flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-trigger-wrapper">
            <label className='text-light text-sm m-0'>
              {selectedContract != null
                ? `${getContractNameFromFullName(selectedContract.contractName)}, ${getShortenedHash(selectedContract.address, 6, 4)}`
                : 'No contract selected'}
            </label>
            <BsChevronDown style={{
              transform: dropdownControl ? 'rotate(180deg)' : 'none',
              transition: 'all 0.3s ease'
            }} />
          </div>
        </D.Trigger>
        <D.Portal>
          <D.Content>
            {contracts.map((contract, index) => {
              return (
                <D.Item
                  key={index}
                  onSelect={() => {
                    setSelectedContract(contract)
                    setDropdownControl(false)
                  }}
                >
                  {`${getContractNameFromFullName(contract.contractName)}, ${getShortenedHash(contract.address, 8, 8)}`}
                </D.Item>
              )
            })}
          </D.Content>
        </D.Portal>
      </D.Root>

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
