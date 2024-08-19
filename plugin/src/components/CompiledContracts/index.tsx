// A component that reads the compiled contracts from the context and displays them in a select
import React, { useState } from 'react'
import { contractsAtom, selectedContractAtom } from '../../atoms/compiledContracts'
import { useAtom, useAtomValue } from 'jotai'
import * as Dropdown from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'
import './compiledContracts.css'

interface CompiledContractsProps {
  show: 'class' | 'contract'
}

const CompiledContracts: React.FC<CompiledContractsProps> = () => {
  const contracts = useAtomValue(contractsAtom)
  const [selectedContract, setSelectedContract] = useAtom(selectedContractAtom)
  const [dropdownControl, setDropdownControl] = useState(false)

  return (
    <div>
      <div className="environment-selector-wrapper">
        <Dropdown.Root
          open={dropdownControl}
          onOpenChange={(e) => {
            setDropdownControl(e)
          }}
        >
          <Dropdown.Trigger>
            <div className="flex flex-row justify-content-space-between align-items-center p-2 pb-1 br-1 compiled-contracts-wrapper">
              <label>{selectedContract?.contractName}</label>
              <BsChevronDown
                style={{
                  transform: dropdownControl ? 'rotate(180deg)' : 'none',
                  transition: 'all 0.3s ease'
                }}
              />
            </div>
          </Dropdown.Trigger>
          <Dropdown.Portal>
            <Dropdown.Content>
              {contracts.map((contract) => {
                return (
                  <Dropdown.Item
                    key={contract.contractName}
                    onClick={() => {
                      setSelectedContract(contract)
                    }}
                  >
                    {contract.contractName}
                  </Dropdown.Item>
                )
              })}
            </Dropdown.Content>
          </Dropdown.Portal>
        </Dropdown.Root>
      </div>
    </div>
  )
}

export default CompiledContracts
