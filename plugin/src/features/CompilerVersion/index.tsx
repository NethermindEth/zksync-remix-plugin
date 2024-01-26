import * as D from '../../ui_components/Dropdown'
import React, { useEffect } from 'react'
import { apiUrl } from '../../utils/network'
import Nethermind from '../../components/NM'
import './style.css'
import { BsChevronDown } from 'react-icons/bs'
import { useAtom } from 'jotai'
import { solidityVersionAtom, versionsAtom } from '../../atoms/version'
import useRemixClient from '../../hooks/useRemixClient'

const SolidityVersion: React.FC = () => {
  const { remixClient } = useRemixClient()

  const envViteVersion: string | undefined = import.meta.env.VITE_VERSION
  const pluginVersion = envViteVersion !== undefined ? `v${envViteVersion}` : 'v0.2.5'

  const [solidityVersion, setSolidityVersion] = useAtom(solidityVersionAtom)
  const [versions, setVersions] = useAtom(versionsAtom)

  const fetchVersions = async (): Promise<void> => {
    try {
      if (apiUrl !== undefined) {
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
      }
    } catch (e) {
      await remixClient.call(
        'notification' as any,
        'toast',
        '🔴 Failed to fetch solidity versions from the compilation server'
      )
      console.error(e)
    }
  }

  // fetch versions
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    const id = setTimeout(fetchVersions, 100)
    return () => {
      clearInterval(id)
    }
  }, [remixClient])

  useEffect(() => {
    if (versions.length === 0) {
      // eslint-disable-next-line @typescript-eslint/no-misused-promises
      const id = setTimeout(fetchVersions, 100)
      return () => {
        clearInterval(id)
      }
    }
  }, [versions])

  return (
    <div className='version-wrapper'>
      <div>
        <D.Root>
          <D.Trigger>
            <label className='solidity-version-legend'>
              Using zksolc-{solidityVersion} <BsChevronDown />
            </label>
          </D.Trigger>
          <D.Portal>
            <D.Content>
              {versions.map((v, i) => {
                return (
                  <D.Item
                    key={i}
                    onClick={() => {
                      setSolidityVersion(v)
                    }}
                  >
                    {v}
                  </D.Item>
                )
              })}
            </D.Content>
          </D.Portal>
        </D.Root>
      </div>
      <div className='version-right'>
        <label className='nethermind-powered'>
          <span style={{ marginRight: '4px' }}>Powered by </span>
          <Nethermind size='xs' />
        </label>
        <label className='plugin-version'>
          Plugin version: {pluginVersion}
        </label>
      </div>
    </div>
  )
}

export default SolidityVersion
