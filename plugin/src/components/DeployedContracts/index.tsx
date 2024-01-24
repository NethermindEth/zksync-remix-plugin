// A component that reads the compiled contracts from the context and displays them in a select

import React, { useEffect } from 'react'
import { getContractNameFromFullName, getShortenedHash } from '../../utils/utils'
import FunctionalInput from '../FunctionalInput'
import './deployedContracts.css'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { deployedContractsAtom, deployedSelectedContractAtom } from '../../atoms/deployedContracts'
import * as D from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'
import copy from 'copy-to-clipboard'
import { MdCopyAll } from 'react-icons/md'
import { FaCheck } from 'react-icons/fa'
import useRemixClient from '../../hooks/useRemixClient'
import { envAtom } from '../../atoms/environment'

// eslint-disable-next-line @typescript-eslint/no-empty-interface

const DeployedContracts: React.FC = () => {
  const
    contracts = useAtomValue(deployedContractsAtom)
  const [
    selectedContract,
    setSelectedContract
  ] = useAtom(deployedSelectedContractAtom)

  const { remixClient } = useRemixClient()

  const [dropdownControl, setDropdownControl] = React.useState(false)

  const setEnv = useSetAtom(envAtom)

  const [copied, setCopied] = React.useState(false)

  useEffect(() => {
    if (copied) {
      remixClient.call(
        'notification' as any,
        'toast',
        '📋 Copied contract address to clipboard'
      ).catch(console.error)
    }
  }, [copied])

  return (
    <>
      <D.Root open={dropdownControl} onOpenChange={(e) => { setDropdownControl(e) }}>
        <div className={'flex flex-row'}>
          <D.Trigger>
            <div className="w-100 flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-trigger-wrapper">
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
        <D.Portal>
          <D.Content>
            {contracts.map((contract, index) => {
              return (
                <D.Item
                  key={index}
                  onSelect={() => {
                    setSelectedContract(contract)
                    setEnv(contract.env)
                    setDropdownControl(false)
                  }}
                >
                  {`[${contract.env}] ${getContractNameFromFullName(contract.contractName)}, ${getShortenedHash(contract.address, 8, 8)}`}
                </D.Item>
              )
            })}
          </D.Content>
        </D.Portal>
      </D.Root>

      {selectedContract?.abi != null
        ? (
          <div>
            {selectedContract.abi.map(
              (abiElement, index) => {
                return abiElement.type === 'function' && (
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
