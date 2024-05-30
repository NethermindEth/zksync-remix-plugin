import React, { useEffect } from 'react'
import { useAtom, useAtomValue } from 'jotai'
import semver from 'semver'
import { BsChevronDown } from 'react-icons/bs'
import * as Dropdown from '../../ui_components/Dropdown'
import { apiUrl } from '../../utils/network'
import Nethermind from '../../components/NM'
import { solidityVersionAtom, versionsAtom } from '../../atoms/version'
import { remixClientAtom } from '../../stores/remixClient'
import './style.css'
import useAsync from '@/hooks/useAsync'
import useAsyncFn from '@/hooks/useAsyncFn'
import useTimeoutFn from '@/hooks/useTimeoutFn'

const envViteVersion: string | undefined = import.meta.env.VITE_VERSION
const pluginVersion = envViteVersion !== undefined ? `v${envViteVersion}` : 'v0.2.5'

const DEFAULT_DELAY = 5_000

export const SolidityVersion: React.FC = () => {
  const remixClient = useAtomValue(remixClientAtom)
  const [solidityVersion, setSolidityVersion] = useAtom(solidityVersionAtom)
  const [versions, setVersions] = useAtom(versionsAtom)

  useAsync(async () => {
    try {
      const response = await fetch(`${apiUrl}/service_version`, {
        method: 'GET',
        redirect: 'follow',
        headers: {
          'Content-Type': 'application/octet-stream'
        }
      })
      const serviceVersion = await response.text()

      if (serviceVersion === 'unknown') {
        await remixClient.call('notification' as any, 'toast', '🔴 Failed to check for updates to the zkSync plugin')
      } else if (semver.gt(serviceVersion, pluginVersion)) {
        await remixClient.call(
          'notification' as any,
          'toast',
          `🔴 You are using an outdated version of the zkSync plugin: ${pluginVersion}, please update to ${serviceVersion} by force-refreshing the page and clearing your browser cache.`
        )

        await remixClient.terminal.log({
          value: `🔴 You are using an outdated version of the zkSync plugin: ${pluginVersion}, please update to ${serviceVersion} by force-refreshing the page and clearing your browser cache.`,
          type: 'error'
        })
      } else {
        await remixClient.call(
          'notification' as any,
          'toast',
          `🟢 You are using the latest version of the zkSync plugin: ${pluginVersion}`
        )

        await remixClient.terminal.log({
          value: `🟢 You are using the latest version of the zkSync plugin: ${pluginVersion}`,
          type: 'info'
        })
      }
    } catch (error) {
      await remixClient.call('notification' as any, 'toast', '🔴 Failed to connect to the compilation server')
      console.error(error)
    }
  }, [remixClient])

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
    <div className="version-wrapper">
      <div>
        <Dropdown.Root>
          <Dropdown.Trigger>
            <label className="solidity-version-legend">
              Using zksolc-{solidityVersion} <BsChevronDown />
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
      <div className="version-right">
        <label className="nethermind-powered">
          <span style={{ marginRight: '4px' }}>Powered by </span>
          <Nethermind size="xs" />
        </label>
        <label className="plugin-version">Plugin version: {pluginVersion}</label>
      </div>
    </div>
  )
}
