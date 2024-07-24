import React from 'react'
import { useAtomValue } from 'jotai'
import semver from 'semver'
import { apiUrl } from '../../utils/network'
import { remixClientAtom } from '../../stores/remixClient'
import './style.css'
import useAsync from '@/hooks/useAsync'

const envViteVersion: string | undefined = import.meta.env.VITE_VERSION
const pluginVersion = envViteVersion !== undefined ? `v${envViteVersion}` : 'v0.2.5'

export const SolidityVersion: React.FC = () => {
  const remixClient = useAtomValue(remixClientAtom)

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

  return (
    <div className="version-wrapper">
      <div className="version-right">
        <label className="plugin-version">Plugin version: {pluginVersion}</label>
      </div>
    </div>
  )
}
