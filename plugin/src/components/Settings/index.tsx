import React from 'react'
import { BsChevronDown } from 'react-icons/bs'
import { useAtom, useAtomValue } from 'jotai'
import { solidityVersionAtom, versionsAtom } from '@/atoms'
import * as Dropdown from '@/ui_components/Dropdown'
import './settings.css'

export const Settings = () => {
  const [solidityVersion, setSolidityVersion] = useAtom(solidityVersionAtom)
  const versions = useAtomValue(versionsAtom)

  return (
    <div className="settings-wrapper">
      <div className="text-center font-bold w-full">Settings</div>
      <div className={'settings-box'}>
        <div className={'settings-box-header'}>zksolc Version</div>
        <div className={'blank'}></div>
        <div className={'settings-box-content'}>
          <div>
            <Dropdown.Root>
              <Dropdown.Trigger>
                <label className="flex p-2 border gap-2">
                  zksolc-{solidityVersion}&nbsp;
                  <BsChevronDown />
                </label>
              </Dropdown.Trigger>
              <Dropdown.Portal>
                <Dropdown.Content>
                  {versions.map((v, i) => {
                    return (
                      <Dropdown.Item
                        key={i}
                        onClick={() => {
                          setSolidityVersion(v)
                        }}
                      >
                        {v}
                      </Dropdown.Item>
                    )
                  })}
                </Dropdown.Content>
              </Dropdown.Portal>
            </Dropdown.Root>
          </div>
        </div>
      </div>
    </div>
  )
}
