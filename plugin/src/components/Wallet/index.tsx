import React, { useEffect } from 'react'
import './wallet.css'
import * as zksync from 'zksync-ethers'
import { useAccount, useWalletClient } from 'wagmi'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { useAtomValue, useSetAtom } from 'jotai'
import { envAtom } from '../../atoms/environment'

const Wallet: React.FC = () => {
  const { data: walletClient } = useWalletClient()
  const setAccount = useSetAtom(accountAtom)
  const setProvider = useSetAtom(providerAtom)
  const env = useAtomValue(envAtom)
  const { isDisconnected } = useAccount()

  useEffect((): void => {
    if (walletClient != null && !isDisconnected) {
      const network = {
        chainId: walletClient.chain.id,
        name: walletClient.chain.name
      }
      const newProvider = new zksync.Web3Provider(
        walletClient.transport,
        network
      )
      const newSigner = newProvider.getSigner(walletClient.account.address)

      setAccount(newSigner)
      setProvider(newProvider)
    }
  }, [
    walletClient?.account.address,
    walletClient?.chain.id,
    env,
    setAccount,
    walletClient,
    isDisconnected,
    setProvider
  ])

  useEffect(() => {
    if (isDisconnected) {
      setAccount(null)
      setProvider(null)
    }
  }, [isDisconnected])

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
