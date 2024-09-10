import React, {useCallback, useState, useEffect} from 'react'
import {useAtom} from 'jotai'
import {customNetworkAtom, isCustomNetworkAliveAtom} from '@/atoms'
import useInterval from '@/hooks/useInterval'
import {remixClient} from '@/PluginClient'
import {FaCheck, FaPlug, FaTimes} from 'react-icons/fa'
import './customNetwork.css'
import {AccountSelector} from '../DevnetAccountSelector'
import {FaRotate} from "react-icons/fa6"

const CUSTOM_NETWORK_POLL_INTERVAL = 10_000

export const CustomNetwork = () => {
    const [customNetwork, setCustomNetwork] = useAtom(customNetworkAtom)
    const [customNetworkInput, setCustomNetworkInput] = useState(customNetwork)
    const [, setIsCustomNetworkAlive] = useAtom(isCustomNetworkAliveAtom)
    const [connectionStatus, setConnectionStatus] = useState('idle') // 'idle', 'connecting', 'connected', 'error'

    const fetchCustomNetworkStatus = useCallback(() => {
        if (customNetwork) {
            setConnectionStatus('connecting')
            fetch(`${customNetwork}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method: 'zks_L1BatchNumber',
                    params: [],
                    id: 1
                })
            })
                .then(async (res) => await res.json())
                .then((res) => {
                    if (res.result) {
                        setIsCustomNetworkAlive(true)
                        setConnectionStatus('connected')
                    } else {
                        remixClient.terminal.log({
                            type: 'error',
                            value: `Failed to connect with RPC Url ${customNetwork}`
                        })
                        setIsCustomNetworkAlive(false)
                        setConnectionStatus('error')
                    }
                })
                .catch((err) => {
                    console.error(err)
                    remixClient.terminal.log({
                        type: 'error',
                        value: `Failed to connect with RPC Url ${customNetwork}`
                    })
                    setIsCustomNetworkAlive(false)
                    setConnectionStatus('error')
                })
        }
    }, [setIsCustomNetworkAlive, customNetwork])

    useInterval(fetchCustomNetworkStatus, customNetwork.length > 0 ? CUSTOM_NETWORK_POLL_INTERVAL : null)

    useEffect(() => {
        if (customNetwork) {
            fetchCustomNetworkStatus()
        } else {
            setConnectionStatus('idle')
        }
    }, [customNetwork, fetchCustomNetworkStatus])

    const handleConnect = () => {
        setCustomNetwork(customNetworkInput)
    }

    const handleDisconnect = () => {
        setCustomNetwork('')
        setConnectionStatus('idle')
        setIsCustomNetworkAlive(false)
    }

    const getButtonClass = () => {
        switch (connectionStatus) {
            case 'not-connected':
                return 'btn-connect-secondary'
            case 'connecting':
                return 'btn-connect-info'
            case 'connected':
                return 'btn-connect-success'
            case 'error':
                return 'btn-connect-danger'
            default:
                return 'btn-connect-secondary'
        }
    }

    const getButtonText = () => {
        switch (connectionStatus) {
            case 'connecting':
                return 'Connecting...'
            case 'connected':
                return 'Disconnect'
            case 'error':
                return 'Retry'
            default:
                return 'Connect'
        }
    }

    return (
        <div
            className="flex"
            style={{
                display: 'flex',
                flexDirection: 'column',
                padding: '1rem 0rem'
            }}
        >
            <div className="custom-network-container">
                <label htmlFor="custom-network-input">Custom Network</label>
                <div className="input-button-wrapper">
                    <input
                        type="text"
                        id="custom-network-input"
                        className="custom-input"
                        placeholder="http://localhost:3050"
                        value={customNetworkInput}
                        onChange={(e) => setCustomNetworkInput(e.target.value)}
                        disabled={connectionStatus === 'connected'}
                    />
                    <button
                        className={`btn-connect ${getButtonClass()}`}
                        onClick={connectionStatus === 'connected' ? handleDisconnect : handleConnect}
                        disabled={connectionStatus === 'connecting'}
                        title={getButtonText()}
                    >
                        <span className="connect-icon">
                            {connectionStatus === 'connected' ? (
                                <FaCheck/>
                            ) : connectionStatus === 'connecting' ? (
                                <span className="spinner-border spinner-border-sm" role="status"
                                      aria-hidden="true"></span>
                            ) : (connectionStatus === 'idle' || connectionStatus == '') ? (
                                <FaPlug/>
                            ) : (
                                <FaRotate/>
                            )}
                        </span>
                        {connectionStatus === 'connected' && (
                            <span className="disconnect-icon">
                                <FaTimes/>
                            </span>
                        )}
                    </button>
                </div>
            </div>
            <AccountSelector accountsType="customNet"/>
        </div>
    )
}
