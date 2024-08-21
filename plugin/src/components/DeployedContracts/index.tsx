import React, { useEffect, useState } from 'react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { BsChevronDown } from 'react-icons/bs'
import copy from 'copy-to-clipboard'
import { MdCopyAll } from 'react-icons/md'
import { FaCheck } from 'react-icons/fa'
import { getContractNameFromFullName, getShortenedHash } from '@/utils/utils'
import FunctionalInput from '@/components/FunctionalInput'
import './deployedContracts.css'
import { deployedContractsAtom, deployedSelectedContractAtom } from '@/atoms/deployedContracts'
import * as Dropdown from '@/ui_components/Dropdown'
import { envAtom } from '@/atoms/environment'
import { remixClientAtom } from '@/stores/remixClient'

const DeployedContracts = () => {
  const contracts = useAtomValue(deployedContractsAtom)
  const [selectedContract, setSelectedContract] = useAtom(deployedSelectedContractAtom)
  const remixClient = useAtomValue(remixClientAtom)
  const [dropdownControl, setDropdownControl] = useState(false)
  const setEnv = useSetAtom(envAtom)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    if (copied) {
      remixClient.call('notification' as any, 'toast', '📋 Copied contract address to clipboard').catch(console.error)
    }
  }, [copied, remixClient])

  return (
    <>
      <Dropdown.Root
        open={dropdownControl}
        onOpenChange={(e) => {
          setDropdownControl(e)
        }}
      >
        <div className="flex flex-row">
          <Dropdown.Trigger>
            <div className="w-100 flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-trigger-wrapper">
              <label className="text-light text-sm m-0">
                {selectedContract != null
                  ? `${getContractNameFromFullName(
                      selectedContract.contractName
                    )}, ${getShortenedHash(selectedContract.address, 6, 4)}`
                  : 'No contract selected'}
              </label>
              <BsChevronDown
                style={{
                  transform: dropdownControl ? 'rotate(180deg)' : 'none',
                  transition: 'all 0.3s ease'
                }}
              />
            </div>
          </Dropdown.Trigger>
          <button
            className="btn"
            onClick={() => {
              copy(selectedContract?.address ?? '')
              setCopied(true)
              setTimeout(() => {
                setCopied(false)
              }, 1000)
            }}
          >
            {copied ? <FaCheck /> : <MdCopyAll />}
          </button>
        </div>
        <Dropdown.Portal>
          <Dropdown.Content>
            {contracts.map((contract) => {
              return (
                <Dropdown.Item
                  key={contract.address}
                  onSelect={() => {
                    setSelectedContract(contract)
                    setEnv(contract.env)
                    setDropdownControl(false)
                  }}
                >
                  <div className="py-1">
                    {`[${contract.env}] ${getContractNameFromFullName(
                      contract.contractName
                    )}, ${getShortenedHash(contract.address, 8, 8)}`}
                  </div>
                </Dropdown.Item>
              )
            })}
          </Dropdown.Content>
        </Dropdown.Portal>
      </Dropdown.Root>

      {selectedContract?.abi != null ? (
        <div>
          {selectedContract.abi
            .filter((element) => element.type === 'function')
            .map((abiElement, index) => (
              <div key={index} className={'methodBox'}>
                <FunctionalInput element={abiElement} />
              </div>
            ))}
        </div>
      ) : (
        <p>No contract selected</p>
      )}
    </>
  )
}

export default DeployedContracts
