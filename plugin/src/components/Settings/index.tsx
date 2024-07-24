import React, { useEffect } from 'react'
import './settings.css'
import { BsChevronDown } from 'react-icons/bs'
import { useAtom, useAtomValue } from 'jotai'
import useAsyncFn from '@/hooks/useAsyncFn'
import { remixClientAtom } from '@/stores/remixClient'
import { apiUrl } from '@/utils/network'
import { solidityVersionAtom, versionsAtom } from '@/atoms'
import * as Dropdown from '@/ui_components/Dropdown'
import useTimeoutFn from '@/hooks/useTimeoutFn'

const DEFAULT_DELAY = 5_000

export const Settings = () => {
  const remixClient = useAtomValue(remixClientAtom)
  const [solidityVersion, setSolidityVersion] = useAtom(solidityVersionAtom)
  const [versions, setVersions] = useAtom(versionsAtom)

  const [, refetchVersions] = useAsyncFn(async () => {
    try {
      await remixClient.call(
        'notification' as any,
        'toast',
        `🟢 Fetching solidity versions from the compilation server at ${apiUrl}`
      )
      const response = await fetch(`${apiUrl}/allowed_versions`, {
        method: 'GET',
        redirect: 'follow',
        headers: {
          'Content-Type': 'application/octet-stream'
        }
      })
      const allowedVersions = await response.json()
      setVersions(allowedVersions)
      if (allowedVersions.length > 0) {
        setSolidityVersion(allowedVersions[0])
      }

      return allowedVersions
    } catch (error) {
      await remixClient.call(
        'notification' as any,
        'toast',
        '🔴 Failed to fetch solidity versions from the compilation server'
      )
      console.error(error)
      throw error
    }
  }, [remixClient])

  const [, cancelRefetchVersions] = useTimeoutFn(refetchVersions, DEFAULT_DELAY)

  useEffect(() => {
    if (versions.length > 0) {
      cancelRefetchVersions()
    }
  }, [versions, cancelRefetchVersions])

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
