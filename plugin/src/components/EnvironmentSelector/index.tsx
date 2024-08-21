import React, { useState } from 'react'
import { useAtom, useSetAtom } from 'jotai'
import { BsChevronDown } from 'react-icons/bs'
import { devnets } from '@/utils/network'
import { devnetAtom, envAtom, providerAtom } from '@/atoms'
import * as Dropdown from '@/ui_components/Dropdown'
import { envName } from '@/utils/misc'
import './styles.css'

export const EnvironmentSelector = () => {
  const [env, setEnv] = useAtom(envAtom)
  const setDevnet = useSetAtom(devnetAtom)
  const setProvider = useSetAtom(providerAtom)
  const [dropdownControl, setDropdownControl] = useState(false)

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
              key={'0wallet'}
              onClick={() => {
                handleEnvironmentChange('0')
              }}
            >
              Wallet
            </Dropdown.Item>

            {devnets.map((devnet, i) => {
              return (
                <Dropdown.Item
                  key={i.toString() + devnet?.name}
                  onClick={() => {
                    handleEnvironmentChange((i + 2).toString())
                  }}
                >
                  {devnet?.name}
                </Dropdown.Item>
              )
            })}
          </Dropdown.Content>
        </Dropdown.Portal>
      </Dropdown.Root>
    </div>
  )
}

export default EnvironmentSelector
