/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import React, { useEffect } from 'react'
import './wallet.css'
import * as zksync from 'zksync-web3'
import { useWalletClient } from 'wagmi'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { useAtomValue, useSetAtom } from 'jotai'
import { envAtom } from '../../atoms/environment'

const Wallet: React.FC = () => {
  const { data: walletClient } = useWalletClient()
  const setAccount = useSetAtom(accountAtom)
  const setProvider = useSetAtom(providerAtom)
  const env = useAtomValue(envAtom)

  useEffect((): void => {
    if (walletClient != null) {
      const network = {
        chainId: walletClient.chain.id,
        name: walletClient.chain.name
      }
      const newProvider = new zksync.Web3Provider(walletClient.transport, network)
      const newSigner = newProvider.getSigner(walletClient.account.address)

      setAccount(newSigner)
      setProvider(newProvider)
    }
  }, [walletClient?.account.address, walletClient?.chain.id, env])

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
