import React, { useCallback, useState } from 'react'
import InputField from '../InputField'
import { useAtom } from 'jotai'
import { customNetworkAtom, isCustomNetworkAliveAtom } from '@/atoms'
import useInterval from '@/hooks/useInterval'
import { remixClient } from '@/PluginClient'
import { AccountSelector } from '../DevnetAccountSelector'
import './customNetwork.css'

const CUSTOMNET_POLL_INTERVAL = 10_000

export const CustomNetwork = () => {
  const [customNetwork, setCustomNetwork] = useAtom(customNetworkAtom)
  const [customNetworkInput, setCustomNetworkInput] = useState(customNetwork)
  const [isCustomNetworkAlive, setIsCustomNetworkAlive] = useAtom(isCustomNetworkAliveAtom)

  const fetchCustomNetworkStatus = useCallback(() => {
    if (customNetwork) {
      fetch(`${customNetwork}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'eth_blockNumber',
          params: [],
          id: 1
        })
      })
        .then(async (res) => await res.json())
        .then((res) => {
          if (res.result) {
            setIsCustomNetworkAlive(true)
          } else {
            remixClient.terminal.log({
              type: 'error',
              value: `Failed to connect with RPC Url ${customNetwork}`
            })
            setIsCustomNetworkAlive(false)
          }
        })
        .catch(() => {
          remixClient.terminal.log({
            type: 'error',
            value: `Failed to connect with RPC Url ${customNetwork}`
          })
          setIsCustomNetworkAlive(false)
        })
    }
  }, [setIsCustomNetworkAlive, customNetwork])

  useInterval(fetchCustomNetworkStatus, customNetwork.length > 0 ? CUSTOMNET_POLL_INTERVAL : null)

  return (
    <div
      className="flex"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
        padding: '1rem 0rem'
      }}
    >
      <div className="custom-network-container">
        <div>
          <InputField
            index={0}
            value={customNetworkInput}
            onChange={(_, newValue) => {
              setCustomNetworkInput(newValue)
              fetchCustomNetworkStatus()
            }}
          />
        </div>{' '}
        <button
          className="btn btn-secondary d-block text-break mb-1 text-center"
          onClick={() => {
            setCustomNetwork(customNetworkInput)
          }}
        >
          {isCustomNetworkAlive ? `Connected` : `Connect`}
        </button>
      </div>
      <AccountSelector accountsType="customNet" />
    </div>
  )
}
