import {
  getRoundedNumber,
  getSelectedAccountIndex,
  getShortenedHash,
  weiToEth
} from '../../utils/utils'
import { getAccounts, updateBalances } from '../../utils/network'
import React, { useContext, useEffect, useState } from 'react'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import { Provider, Wallet } from 'zksync-web3'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import { MdCopyAll, MdRefresh } from 'react-icons/md'
import './devnetAccountSelector.css'
import EnvironmentContext from '../../contexts/EnvironmentContext'
import copy from 'copy-to-clipboard'
import TransactionContext from '../../contexts/TransactionContext'

const DevnetAccountSelector: React.FC = () => {
  const { account, setAccount, provider, setProvider } = useContext(ConnectionContext)
  const remixClient = useContext(RemixClientContext)
  const {
    env,
    devnet,
    isDevnetAlive,
    setIsDevnetAlive,
    selectedDevnetAccount,
    setSelectedDevnetAccount,
    availableDevnetAccounts,
    setAvailableDevnetAccounts
  } = useContext(EnvironmentContext)

  const { transactions } = useContext(TransactionContext)

  const [accountRefreshing, setAccountRefreshing] = useState(false)
  const [showCopied, setCopied] = useState(false)

  const [accountIdx, setAccountIdx] = useState(0)

  // devnet live status
  useEffect(() => {
    let isSubscribed = true

    const interval = setInterval(async () => {
      try {
        const response = await fetch(`${devnet.url}`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ jsonrpc: '2.0', method: 'eth_blockNumber', params: [], id: 1 })
        })

        if (response.status === 200) {
          const responseBody = await response.json()
          if (responseBody.result != null && isSubscribed) {
            setIsDevnetAlive(true)
          } else if (isSubscribed) {
            setIsDevnetAlive(false)
          }
        } else if (isSubscribed) {
          setIsDevnetAlive(false)
        }
      } catch (error) {
        if (isSubscribed) {
          setIsDevnetAlive(false)
        }
      }
    }, 1000)

    return () => {
      clearInterval(interval)
      isSubscribed = false
    }
  }, [devnet.url])

  const notifyDevnetStatus = async (): Promise<void> => {
    try {
      await remixClient.call(
        'notification' as any,
        'toast',
        `❗️ Server ${devnet.name} - ${devnet.url} is not healthy or not reachable at the moment`
      )
    } catch (e) {
      console.log(e)
    }
  }

  useEffect(() => {
    updateAccountBalances().catch((e) => {
      console.log(e)
    })
  }, [transactions])

  useEffect(() => {
    if (!isDevnetAlive) {
      notifyDevnetStatus().catch((e) => {
        console.log(e)
      })
    }
  }, [isDevnetAlive])

  const updateAccountBalances = async (): Promise<void> => {
    const updatedAccounts = await updateBalances(availableDevnetAccounts)
    setAvailableDevnetAccounts(updatedAccounts)
  }
  const refreshDevnetAccounts = async (): Promise<void> => {
    setAccountRefreshing(true)
    try {
      const accounts = await getAccounts(`${devnet.url}`)
      if (
        JSON.stringify(accounts) !== JSON.stringify(availableDevnetAccounts)
      ) {
        setAvailableDevnetAccounts(accounts)
      }
    } catch (e) {
      await remixClient.terminal.log({
        type: 'error',
        value: `Failed to get accounts information from ${devnet.url}`
      })
    }
    setAccountRefreshing(false)
  }

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      // if (!isDevnetAlive) {
      //   return
      // }
      await refreshDevnetAccounts()
    }, 1)
  }, [devnet])

  useEffect(() => {
    if (
      !(
        selectedDevnetAccount !== null &&
        availableDevnetAccounts.includes(selectedDevnetAccount)
      ) &&
      availableDevnetAccounts.length > 0
    ) {
      setSelectedDevnetAccount(availableDevnetAccounts[0])
    }
  }, [availableDevnetAccounts, devnet])

  useEffect(() => {
    const newProvider = new Provider(devnet.url)
    if (selectedDevnetAccount != null) {
      setAccount(
        new Wallet(
          selectedDevnetAccount.private_key,
          newProvider
        )
      )
    }
    setProvider(newProvider)
  }, [devnet, selectedDevnetAccount])

  function handleAccountChange (event: any): void {
    if (event.target.value === -1) {
      return
    }
    setAccountIdx(event.target.value)
    setSelectedDevnetAccount(availableDevnetAccounts[event.target.value])
    const newProvider = new Provider(devnet.url)
    if (provider == null) setProvider(newProvider)
    setAccount(
      new Wallet(
        availableDevnetAccounts[event.target.value].private_key,
        provider ?? newProvider

      )
    )
  }

  useEffect(() => {
    setAccountIdx(0)
  }, [env])
  return (
    <>
      <label className="">Devnet account selection</label>
      <div className="devnet-account-selector-wrapper">
        <select
          className="custom-select"
          aria-label=".form-select-sm example"
          onChange={handleAccountChange}
          value={accountIdx}
          defaultValue={getSelectedAccountIndex(
            availableDevnetAccounts,
            selectedDevnetAccount
          )}
        >
          {isDevnetAlive && availableDevnetAccounts.length > 0
            ? availableDevnetAccounts.map((account, index) => {
              return (
                  <option value={index} key={index}>
                    {`${getShortenedHash(
                      account.address ?? '',
                      6,
                      4
                    )}
                    (${getRoundedNumber(weiToEth(Number(account.initial_balance)), 2)} ether)`
                    }
                  </option>
              )
            })
            : ([
                <option value={-1} key={-1}>
                  No accounts found
                </option>
              ] as JSX.Element[])}
        </select>
        <div className="position-relative">
          <button
            className="btn"
            onClick={() => {
              copy(account?.address ?? '')
              setCopied(true)
              setTimeout(() => {
                setCopied(false)
              }, 1000)
            }}
          >
            <MdCopyAll />
          </button>
          {showCopied && (
            <p className="position-absolute text-copied">Copied</p>
          )}
        </div>
        <button
          className="btn refresh"
          // eslint-disable-next-line @typescript-eslint/no-misused-promises
          onClick={refreshDevnetAccounts}
          title="Refresh devnet accounts"
          data-loading={accountRefreshing ? 'loading' : 'loaded'}
        >
          <MdRefresh />
        </button>
      </div>
    </>
  )
}

export default DevnetAccountSelector
