/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import {
  useConnectModal,
  useAccountModal,
  useChainModal,
  ConnectButton
} from '@rainbow-me/rainbowkit';
import React, { ComponentProps, useEffect, useState } from 'react';
import {
  useAccount,
  useNetwork,
  usePrepareSendTransaction,
  useSendTransaction,
  useSignMessage,
  useSignTypedData,
} from 'wagmi';
import './wallet.css'
import { useConnect } from 'wagmi'

type ConnectButtonProps = ComponentProps<typeof ConnectButton>;
type ExtractString<Value> = Value extends string ? Value : never;
type AccountStatus = ExtractString<ConnectButtonProps['accountStatus']>;
type ChainStatus = ExtractString<ConnectButtonProps['chainStatus']>;

const Wallet = () => {
  // const { openConnectModal } = useConnectModal();
  // const { openAccountModal } = useAccountModal();
  // const { openChainModal } = useChainModal();
  const { openAccountModal, accountModalOpen } = useAccountModal();
  const { openChainModal, chainModalOpen } = useChainModal();
  const { openConnectModal, connectModalOpen } = useConnectModal();
  const { address, isConnected} = useAccount();
  const { connect, connectors, error, isLoading, pendingConnector } = useConnect()


  // const { status } = useSession();
  const account = useAccount({
    onConnect({ address, connector, isReconnected }) {
      console.log('Connected', { address, connector, isReconnected })
    },
  })
  console.log(connectors) 
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
              if(address!==account?.address){
                return (
                  <button onClick={openConnectModal} 
                  style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }} 
                  className="btn btn-primary w-100" 
                  type="button">
                    Connect
                  </button>
                );
              }

              if (!connected) {
                return (
                  <button onClick={openConnectModal} 
                  style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }} 
                  className="btn btn-primary w-100" 
                  type="button">
                    Connect
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
                <div style={{ display: 'flex', gap: 12 }}>
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
