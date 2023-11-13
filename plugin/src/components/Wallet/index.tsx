/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useAccount, useNetwork } from 'wagmi'
import React, { useContext, useEffect, useState } from 'react'

import copy from 'copy-to-clipboard'
import './wallet.css'
import * as zksync from 'zksync-web3'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import { useWalletClient } from 'wagmi'
import { MdCopyAll } from 'react-icons/md'
import {  trimStr } from '../../utils/utils'


const Wallet = () => {
  const {isConnected} = useAccount()
  const { data: walletClient } = useWalletClient()
  const { setAccount, setProvider } = useContext(ConnectionContext)
  const chainConnected = useNetwork()
  const [showCopied, setCopied] = useState(false)

  useEffect(() =>  {
    if (walletClient ){
      const network = {
        chainId: walletClient.chain.id,
        name: walletClient.chain.name,
      }
      const newprovider = new zksync.Web3Provider(walletClient.transport, network)
      const newsigner = newprovider.getSigner(walletClient.account.address)

      setAccount(newsigner)
      setProvider(walletClient)
    }

  }, [walletClient?.account.address])

  console.log(isConnected)
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
      <div className="wallet-actions">
      <ConnectButton.Custom>
      {({
        account,
        chain,
        openAccountModal,
        openChainModal,
        openConnectModal,
        authenticationStatus,
        mounted,
      }) => {
        // Note: If your app doesn't use authentication, you
        // can remove all 'authenticationStatus' checks
        const ready = mounted && authenticationStatus !== 'loading';
        const connected =
          ready &&
          account &&
          chain &&
          (!authenticationStatus ||
            authenticationStatus === 'authenticated');

        return (
          <div
            {...(!ready && {
              'aria-hidden': true,
              'style': {
                opacity: 0,
                pointerEvents: 'none',
                userSelect: 'none',
              },
            })}
          >
            {(() => {
              if (!connected) {
                return (
                  <button onClick={openConnectModal} type="button">
                    Connect Wallet
                  </button>
                );
              }

              if (chain.unsupported) {
                return (
                  <button onClick={openChainModal} type="button">
                    Wrong network
                  </button>
                );
              }

              return (
                <div style={{ display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: '1rem',
                            }}>
                <div style={{ display: 'flex',
                            flexDirection: 'row',
                            gap: '1rem',
                            alignItems: 'center',
                            }} >
                  <button
                    onClick={openChainModal}
                    style={{ display: 'flex', alignItems: 'center' }}
                    type="button"
                  >
                    {chain.hasIcon && (
                      <div
                        style={{
                          background: chain.iconBackground,
                          width: 12,
                          height: 12,
                          borderRadius: 999,
                          overflow: 'hidden',
                          marginRight: 4,
                        }}
                      >
                        {chain.iconUrl && (
                          <img
                            alt={chain.name ?? 'Chain icon'}
                            src={chain.iconUrl}
                            style={{ width: 12, height: 12 }}
                          />
                        )}
                      </div>
                    )}
                    {chain.name}
                  </button>

                  <button onClick={openAccountModal} type="button">
                    {account.displayName}
                    {account.displayBalance
                      ? ` (${account.displayBalance})`
                      : ''}
                  </button>
                </div>

                <div className="wallet-account-wrapper">
                    <p
                      className="text account"
                      title={account.address}
                    >
                      <a
                        href={`${chainConnected.chain?.blockExplorers?.default.url}/address/${account.address as string ?? ''}`}
                        target="_blank"
                        rel="noreferer noopener noreferrer"
                      >
                        {trimStr(
                          account.address ?? '',
                          10
                        )}
                      </a>
                    </p>
                    <span style={{ position: 'relative' }}>
                      <button
                        className="btn p-0"
                        onClick={() => {
                          copy(account.address ?? '')
                          setCopied(true)
                          setTimeout(() => {
                            setCopied(false)
                          }, 1000)
                        }}
                      >
                        <MdCopyAll />
                      </button>
                      {showCopied && (
                        <p style={{ position: 'absolute', right: 0, minWidth: '70px' }}>
                          Copied
                        </p>
                      )}
                    </span>
                  </div>
                </div>
              );
            })()}
          </div>
        );
      }}
    </ConnectButton.Custom>
      </div>
    </div>
  )
}

export default Wallet
