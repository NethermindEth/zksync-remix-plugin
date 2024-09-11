import React, { useState } from 'react'
import { useAtom, useSetAtom } from 'jotai'
import { BsChevronDown } from 'react-icons/bs'
import { devnets } from '@/utils/network'
import { devnetAtom, envAtom, providerAtom } from '@/atoms'
import * as Dropdown from '@/ui_components/Dropdown'
import { envName } from '@/utils/misc'
import './styles.css'

const [localDevnet, remoteDevnet] = devnets

export const EnvironmentSelector = () => {
  const [env, setEnv] = useAtom(envAtom)
  const setDevnet = useSetAtom(devnetAtom)
  const setProvider = useSetAtom(providerAtom)
  const [dropdownControl, setDropdownControl] = useState(false)

  const handleEnvironmentChange = (env: string): void => {
    switch (env) {
      case 'localDevnet':
      case 'remoteDevnet': {
        const devnet = env === 'localDevnet' ? localDevnet : remoteDevnet
        setDevnet(devnet)
        setEnv(env)
        setProvider(null)
        break
      }
      case 'wallet':
        setEnv('wallet')
        break
      case 'customNetwork':
        setEnv('customNetwork')
        break
      default:
    }
  }

  return (
    <div className="environment-selector-wrapper">
      <Dropdown.Root
        open={dropdownControl}
        onOpenChange={(e) => {
          setDropdownControl(e)
        }}
      >
        <Dropdown.Trigger>
          <div className="flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-trigger-wrapper">
            <label className="text-light text-sm m-0">{envName(env)}</label>
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
            <Dropdown.Item
              key={'wallet'}
              onClick={() => {
                handleEnvironmentChange('wallet')
              }}
            >
              Wallet
            </Dropdown.Item>
            <Dropdown.Item
              key={localDevnet?.id}
              onClick={() => {
                handleEnvironmentChange(localDevnet.id)
              }}
            >
              {localDevnet?.name}
            </Dropdown.Item>
            <Dropdown.Item
              key={remoteDevnet?.id}
              onClick={() => {
                handleEnvironmentChange(remoteDevnet.id)
              }}
            >
              {remoteDevnet?.name}
            </Dropdown.Item>
            <Dropdown.Item
              key={'customNetwork'}
              onClick={() => {
                handleEnvironmentChange('customNetwork')
              }}
            >
              Custom Network
            </Dropdown.Item>
          </Dropdown.Content>
        </Dropdown.Portal>
      </Dropdown.Root>
    </div>
  )
}

export default EnvironmentSelector
