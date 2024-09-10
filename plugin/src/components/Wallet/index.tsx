import React, { useEffect } from 'react'
import * as zksync from 'zksync-ethers'
import { useAtomValue, useSetAtom } from 'jotai'
import { useAccount, useWalletClient } from 'wagmi'
import { accountAtom, providerAtom, accountInfoAtom } from '@/atoms/connection'
import { envAtom } from '@/atoms/environment'
import './wallet.css'

export const Wallet = () => {
  const { data: walletClient } = useWalletClient()
  const setAccount = useSetAtom(accountAtom)
  const setProvider = useSetAtom(providerAtom)
  const setAccountInfo = useSetAtom(accountInfoAtom)
  const env = useAtomValue(envAtom)
  const { isDisconnected, address } = useAccount()

  useEffect((): void => {
    if (walletClient != null && !isDisconnected && address) {
      const network = {
        chainId: walletClient.chain.id,
        name: walletClient.chain.name
      }
      const newProvider = new zksync.Web3Provider(walletClient.transport, network)
      const newSigner = newProvider.getSigner(address)
      setAccount(newSigner)
      setProvider(newProvider)
      setAccountInfo({ address, balance: 0 })
    }
  }, [
    walletClient?.account.address,
    walletClient?.chain.id,
    env,
    setAccount,
    walletClient,
    isDisconnected,
    setProvider,
    setAccountInfo,
    address
  ])

  useEffect(() => {
    if (isDisconnected) {
      setAccount(null)
      setProvider(null)
    }
  }, [isDisconnected, setAccount, setProvider])

  return (
    <div
      className="flex wallet-actions"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
        padding: '1rem 0rem'
      }}
    >
      <w3m-button />
    </div>
  )
}

export default Wallet
