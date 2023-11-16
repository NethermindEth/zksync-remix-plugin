import { getRoundedNumber, getShortenedHash, weiToEth } from '../../utils/utils'
import { getAccounts, updateBalances } from '../../utils/network'
import React, { useEffect, useState } from 'react'
import { Provider, Wallet } from 'zksync-web3'
import { MdCopyAll, MdRefresh } from 'react-icons/md'
import './devnetAccountSelector.css'
import copy from 'copy-to-clipboard'
import { useAtom, useAtomValue } from 'jotai'
import { accountAtom, providerAtom } from '../../atoms/connection'
import useRemixClient from '../../hooks/useRemixClient'
import {
  availableDevnetAccountsAtom,
  devnetAtom,
  envAtom,
  isDevnetAliveAtom,
  selectedDevnetAccountAtom
} from '../../atoms/environment'
import { transactionsAtom } from '../../atoms/transaction'
import { BsCheck, BsChevronDown } from 'react-icons/bs'
import * as D from '../../ui_components/Dropdown'
import { FaCheck } from 'react-icons/fa'

const DevnetAccountSelector: React.FC = () => {
  const { remixClient } = useRemixClient()

  const [account, setAccount] = useAtom(accountAtom)
  const [provider, setProvider] = useAtom(providerAtom)

  const env = useAtomValue(envAtom)
  const devnet = useAtomValue(devnetAtom)
  const [isDevnetAlive, setIsDevnetAlive] = useAtom(isDevnetAliveAtom)
  const [selectedDevnetAccount, setSelectedDevnetAccount] = useAtom(selectedDevnetAccountAtom)
  const [availableDevnetAccounts, setAvailableDevnetAccounts] = useAtom(availableDevnetAccountsAtom)

  const transactions = useAtomValue(transactionsAtom)

  const [accountRefreshing, setAccountRefreshing] = useState(false)
  const [showCopied, setCopied] = useState(false)

  const [accountIdx, setAccountIdx] = useState(0)

  // devnet live status
  useEffect(() => {
    let isSubscribed = true

    const interval = setInterval(() => {
      (async () => {
        try {
          const response = await fetch(`${devnet.url}`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'eth_blockNumber',
              params: [],
              id: 1
            })
          })

          const isAlive = isSubscribed && response.status === 200 && (await response.json()).result != null
          setIsDevnetAlive(isAlive)
        } catch (error) {
          if (isSubscribed) {
            setIsDevnetAlive(false)
          }
        }
      })().catch(console.error)
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

  function handleAccountChange (index: number): void {
    if (index === -1) {
      return
    }
    setAccountIdx(index)
    setSelectedDevnetAccount(availableDevnetAccounts[index])
    const newProvider = new Provider(devnet.url)
    if (provider == null) setProvider(newProvider)
    setAccount(
      new Wallet(
        availableDevnetAccounts[index].private_key,
        provider ?? newProvider
      )
    )
  }

  const [dropdownControl, setDropdownControl] = useState(false)

  useEffect(() => {
    setAccountIdx(0)
  }, [env])
  return (
    <div className='mt-2'>
      <label className="">Devnet account selection</label>
      <div className="devnet-account-selector-wrapper">
        <D.Root open={dropdownControl} onOpenChange={(e) => { setDropdownControl(e) }}>
          <D.Trigger >
            <div className='flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-account-selector-trigger'>
              <label className='text-light text-sm m-0'>{(availableDevnetAccounts.length !== 0 && (availableDevnetAccounts[accountIdx]?.address) !== undefined)
                ? getShortenedHash(
                  availableDevnetAccounts[accountIdx]?.address,
                  6,
                  4
                )
                : 'No accounts found'}</label>
              <BsChevronDown style={{
                transform: dropdownControl ? 'rotate(180deg)' : 'none',
                transition: 'all 0.3s ease'
              }} />            </div>
          </D.Trigger>
          <D.Portal>
            <D.Content>
              {isDevnetAlive && availableDevnetAccounts.length > 0
                ? availableDevnetAccounts.map((account, index) => {
                  return (
                    <D.Item onClick={() => { handleAccountChange(index) }} key={index}>
                      {accountIdx === index && <BsCheck size={18} />}
                      {`${getShortenedHash(
                        account.address ?? '',
                        6,
                        4
                      )} (${getRoundedNumber(
                        weiToEth(account.initial_balance),
                        2
                      )} ether)`}
                    </D.Item>
                  )
                })
                : ([
                  <D.Item onClick={() => { handleAccountChange(-1) }} key={-1}>
                    No accounts found
                  </D.Item>
                  ] as JSX.Element[])}
            </D.Content>
          </D.Portal>
        </D.Root>
        <div className="position-relative">
          <button
            className="btn"
            onClick={() => {
              copy((account as Wallet)?.address ?? '')
              setCopied(true)
              setTimeout(() => {
                setCopied(false)
              }, 1000)
            }}
          >
            {showCopied ? <FaCheck /> : <MdCopyAll />}
          </button>
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
    </div>
  )
}

export default DevnetAccountSelector
