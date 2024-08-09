import React, { useState } from 'react'
import { devnets } from '../../utils/network'

import './styles.css'
import { useAtom, useSetAtom } from 'jotai'
import { devnetAtom, envAtom } from '../../atoms/environment'
import { providerAtom } from '../../atoms/connection'
import * as D from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'

export const EnvironmentSelector: React.FC = () => {
  const [env, setEnv] = useAtom(envAtom)
  const setDevnet = useSetAtom(devnetAtom)
  const setProvider = useSetAtom(providerAtom)

  const handleEnvironmentChange = (ipValue: string): void => {
    const value = parseInt(ipValue)
    if (!isNaN(value) && value > 1) {
      setDevnet(devnets[value - 2])
      if (value === 3) {
        setEnv('remoteDevnet')
      } else if (value === 2) {
        setEnv('localDevnet')
      }
      setProvider(null)
    } else if (value === 0) {
      setEnv('wallet')
    } else {
      setEnv('manual')
    }
  }

  const getActiveEnv = (lEnv: typeof env): string => {
    switch (lEnv) {
      case 'manual':
        return 'Manual'
      case 'localDevnet':
        return 'Local Devnet'
      case 'remoteDevnet':
        return 'Remote Devnet'
      case 'wallet':
        return 'Wallet'
    }
  }

  const [dropdownControl, setDropdownControl] = useState(false)

  return (
    <div className="environment-selector-wrapper">
      <D.Root
        open={dropdownControl}
        onOpenChange={(e) => {
          setDropdownControl(e)
        }}
      >
        <D.Trigger>
          <div className="flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-trigger-wrapper">
            <label className="text-light text-sm m-0">{getActiveEnv(env)}</label>
            <BsChevronDown
              style={{
                transform: dropdownControl ? 'rotate(180deg)' : 'none',
                transition: 'all 0.3s ease'
              }}
            />
          </div>
        </D.Trigger>
        <D.Portal>
          <D.Content>
            <D.Item
              key={'0wallet'}
              onClick={() => {
                handleEnvironmentChange('0')
              }}
            >
              Wallet
            </D.Item>
            <D.Item
              key={'1manual'}
              onClick={() => {
                handleEnvironmentChange('1')
              }}
            >
              Manual
            </D.Item>
            {devnets.map((devnet, i) => {
              return (
                <D.Item
                  key={i.toString() + devnet?.name}
                  onClick={() => {
                    handleEnvironmentChange((i + 2).toString())
                  }}
                >
                  {devnet?.name}
                </D.Item>
              )
            })}
          </D.Content>
        </D.Portal>
      </D.Root>
    </div>
  )
}

export default EnvironmentSelector
