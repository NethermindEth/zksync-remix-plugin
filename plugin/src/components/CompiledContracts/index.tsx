// A component that reads the compiled contracts from the context and displays them in a select

import React, { useState } from 'react'
import { contractsAtom, selectedContractAtom } from '../../atoms/compiledContracts'
import { useAtom, useAtomValue } from 'jotai'
import * as D from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'
import './compiledContracts.css'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  show: 'class' | 'contract'
}

const CompiledContracts: React.FC<CompiledContractsProps> = (props) => {
  const
    contracts = useAtomValue(contractsAtom)
  const [
    selectedContract,
    setSelectedContract
  ] = useAtom(selectedContractAtom)

  const [dropdownControl, setDropdownControl] = useState(false)

  return (
    <div>
      <div className="environment-selector-wrapper">
        <D.Root open={dropdownControl} onOpenChange={(e) => { setDropdownControl(e) }}>
          <D.Trigger>
            <div className="flex flex-row justify-content-space-between align-items-center p-2 pb-1 br-1 compiled-contracts-wrapper">
              <label>{selectedContract?.contractName}</label>
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
                    onClick={() => {
                      setSelectedContract(contracts[index])
                    }}
                  >
                    {contract.contractName}
                  </D.Item>
                )
              })}
            </D.Content>
          </D.Portal>
        </D.Root>
      </div>
    </div>
  )
}

export default CompiledContracts
