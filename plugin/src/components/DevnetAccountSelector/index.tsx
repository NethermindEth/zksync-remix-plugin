import React, { useEffect, useState } from 'react'
import { Provider, Wallet } from 'zksync-ethers'
import { MdCopyAll, MdRefresh } from 'react-icons/md'
import copy from 'copy-to-clipboard'
import { useAtom, useAtomValue } from 'jotai'
import { FaCheck } from 'react-icons/fa'
import { BsCheck, BsChevronDown } from 'react-icons/bs'
import { getRoundedNumber, getShortenedHash, weiToEth } from '@/utils/utils'
import { getAccounts, updateBalances } from '@/utils/network'
import { accountAtom, providerAtom } from '@/atoms/connection'
import {
  availableDevnetAccountsAtom,
  devnetAtom,
  envAtom,
  isDevnetAliveAtom,
  selectedDevnetAccountAtom
} from '@/atoms/environment'
import * as Dropdown from '@/ui_components/Dropdown'
import { remixClientAtom } from '@/stores/remixClient'
import useInterval from '@/hooks/useInterval'
import useAsync from '@/hooks/useAsync'
import './devnetAccountSelector.css'
import useAsyncFn from '@/hooks/useAsyncFn'

const DEVNET_POLL_INTERVAL = 10_000

export const DevnetAccountSelector = () => {
  const remixClient = useAtomValue(remixClientAtom)
  const [account, setAccount] = useAtom(accountAtom)
  const [provider, setProvider] = useAtom(providerAtom)

  const devnet = useAtomValue(devnetAtom)
  const [isDevnetAlive, setIsDevnetAlive] = useAtom(isDevnetAliveAtom)
  const [selectedDevnetAccount, setSelectedDevnetAccount] = useAtom(selectedDevnetAccountAtom)
  const [availableDevnetAccounts, setAvailableDevnetAccounts] = useAtom(availableDevnetAccountsAtom)
  const [accountRefreshing, setAccountRefreshing] = useState(false)
  const [showCopied, setCopied] = useState(false)
  const [accountIdx, setAccountIdx] = useState(0)
  const env = useAtomValue(envAtom)

  useInterval(
    () => {
      fetch(`${devnet.url}`, {
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
        .then(async (res) => await res.json())
        .then((res) => {
          if (res.result) {
            setIsDevnetAlive(true)
          } else {
            setIsDevnetAlive(false)
          }
        })
        .catch(() => {
          setIsDevnetAlive(false)
        })
    },
    devnet.url.length > 0 ? DEVNET_POLL_INTERVAL : null
  )

  useEffect(() => {
    if (!isDevnetAlive) {
      remixClient
        .call(
          'notification' as any,
          'toast',
          `❗️ Server ${devnet.name} - ${devnet.url} is not healthy or not reachable at the moment`
        )
        .catch((e) => {
          console.error(e)
        })
    }
  }, [isDevnetAlive, remixClient, devnet])

  useAsync(async () => {
    const updatedAccounts = await updateBalances(availableDevnetAccounts, devnet.url)
    setAvailableDevnetAccounts(updatedAccounts)
  }, [devnet, env])

  const [, refreshDevnetAccounts] = useAsyncFn(async () => {
    try {
      setAccountRefreshing(true)
      const accounts = await getAccounts(`${devnet.url}`)
      if (JSON.stringify(accounts) !== JSON.stringify(availableDevnetAccounts)) {
        setAvailableDevnetAccounts(accounts)
      }
    } catch (error) {
      await remixClient.terminal.log({
        type: 'error',
        value: `Failed to get accounts information from ${devnet.url}`
      })
    }
    setAccountRefreshing(false)
  }, [remixClient, devnet])

  useEffect(() => {
    refreshDevnetAccounts()
  }, [refreshDevnetAccounts])

  useEffect(() => {
    if (
      !(selectedDevnetAccount !== null && availableDevnetAccounts.includes(selectedDevnetAccount)) &&
      availableDevnetAccounts.length > 0 &&
      isDevnetAlive
    ) {
      setSelectedDevnetAccount(availableDevnetAccounts[0])
    }
    if (!isDevnetAlive && selectedDevnetAccount !== null) {
      setSelectedDevnetAccount(null)
    }
  }, [availableDevnetAccounts, devnet, selectedDevnetAccount, setSelectedDevnetAccount, env, isDevnetAlive])

  useEffect(() => {
    const newProvider = new Provider(devnet.url)
    if (selectedDevnetAccount != null) {
      setAccount(new Wallet(selectedDevnetAccount.private_key, newProvider))
    }
    setProvider(newProvider)
  }, [devnet, selectedDevnetAccount, setAccount, setProvider])

  function handleAccountChange(index: number): void {
    if (index === -1) {
      return
    }
    setAccountIdx(index)
    setSelectedDevnetAccount(availableDevnetAccounts[index])
    const newProvider = new Provider(devnet.url)
    if (provider == null) setProvider(newProvider)
    setAccount(new Wallet(availableDevnetAccounts[index].private_key, provider ?? newProvider))
  }

  const [dropdownControl, setDropdownControl] = useState(false)

  useEffect(() => {
    setAccountIdx(0)
  }, [])

  return (
    <div className="mt-2">
      <label className="">Devnet account selection</label>
      <div className="devnet-account-selector-wrapper">
        <Dropdown.Root
          open={dropdownControl}
          onOpenChange={(e) => {
            setDropdownControl(e)
          }}
        >
          <Dropdown.Trigger>
            <div className="flex flex-row justify-content-space-between align-items-center p-2 br-1 devnet-account-selector-trigger">
              <label className="text-light text-sm m-0">
                {isDevnetAlive &&
                availableDevnetAccounts.length !== 0 &&
                availableDevnetAccounts[accountIdx]?.address !== undefined
                  ? getShortenedHash(availableDevnetAccounts[accountIdx]?.address, 6, 4)
                  : 'No accounts found'}
              </label>
              <BsChevronDown
                style={{
                  transform: dropdownControl ? 'rotate(180deg)' : 'none',
                  transition: 'all 0.3s ease'
                }}
              />
            </div>
          </Dropdown.Trigger>
          <Dropdown.Portal>
            <Dropdown.Content>
              {isDevnetAlive && availableDevnetAccounts.length > 0
                ? availableDevnetAccounts.map((account, index) => {
                    return (
                      <Dropdown.Item
                        onClick={() => {
                          handleAccountChange(index)
                        }}
                        key={index}
                      >
                        {accountIdx === index && <BsCheck size={18} />}
                        {`${getShortenedHash(account.address ?? '', 6, 4)} (${getRoundedNumber(
                          weiToEth(account.initial_balance),
                          2
                        )} ether)`}
                      </Dropdown.Item>
                    )
                  })
                : ([
                    <Dropdown.Item
                      onClick={() => {
                        handleAccountChange(-1)
                      }}
                      key={-1}
                    >
                      No accounts found
                    </Dropdown.Item>
                  ] as JSX.Element[])}
            </Dropdown.Content>
          </Dropdown.Portal>
        </Dropdown.Root>
        <div>
          <button
            className="refresh"
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
