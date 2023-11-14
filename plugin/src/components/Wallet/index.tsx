/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import { useAccount, useNetwork } from 'wagmi'
import React, { useContext, useEffect, useState } from 'react'

import './wallet.css'
import * as zksync from 'zksync-web3'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import { useWalletClient } from 'wagmi'


const Wallet = () => {
  const {isConnected} = useAccount()
  const { data: walletClient } = useWalletClient()
  const { setAccount, setProvider } = useContext(ConnectionContext)

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
      <w3m-button />
      </div>
    </div>
  )
}

export default Wallet
