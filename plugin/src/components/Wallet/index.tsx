/* eslint-disable multiline-ternary */
/* eslint-disable @typescript-eslint/no-misused-promises */
import  { useContext, useEffect } from 'react'
import './wallet.css'
import * as zksync from 'zksync-web3'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import { useWalletClient } from 'wagmi'


const Wallet = () => {
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

  return (
      <w3m-button />
  )
}

export default Wallet
