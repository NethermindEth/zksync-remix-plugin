/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import { useContext, useEffect } from 'react'
import './wallet.css'
import * as zksync from 'zksync-web3'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import { useWalletClient } from 'wagmi'
import EnvironmentContext from '../../contexts/EnvironmentContext'

const Wallet = () => {
  const { data: walletClient } = useWalletClient()
  const { setAccount, setProvider } = useContext(ConnectionContext)
  const { env } = useContext(EnvironmentContext)

  useEffect(() => {
    if (walletClient != null) {
      const network = {
        chainId: walletClient.chain.id,
        name: walletClient.chain.name
      }
      const newprovider = new zksync.Web3Provider(walletClient.transport, network)
      const newsigner = newprovider.getSigner(walletClient.account.address)
      setAccount(newsigner)
      setProvider(walletClient)
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
