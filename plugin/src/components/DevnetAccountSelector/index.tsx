import React, { useCallback, useEffect, useState } from 'react'
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
  customNetworkAtom,
  devnetAtom,
  envAtom,
  isCustomNetworkAliveAtom,
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

export const AccountSelector = ({ accountsType }: { accountsType: 'devnet' | 'customNet' }) => {
  const remixClient = useAtomValue(remixClientAtom)
  const [account, setAccount] = useAtom(accountAtom)
  const [provider, setProvider] = useAtom(providerAtom)

  const devnet = useAtomValue(devnetAtom)
  const [isDevnetAlive, setIsDevnetAlive] = useAtom(isDevnetAliveAtom)
  const customNetwork = useAtomValue(customNetworkAtom)
  const isCustomNetworkAlive = useAtomValue(isCustomNetworkAliveAtom)
  const [selectedDevnetAccount, setSelectedDevnetAccount] = useAtom(selectedDevnetAccountAtom)
  const [availableDevnetAccounts, setAvailableDevnetAccounts] = useAtom(availableDevnetAccountsAtom)
  const [accountRefreshing, setAccountRefreshing] = useState(false)
  const [showCopied, setCopied] = useState(false)
  const [accountIdx, setAccountIdx] = useState(0)
  const env = useAtomValue(envAtom)

  const isAlive = accountsType === 'devnet' ? isDevnetAlive : isCustomNetworkAlive
  const networkUrl = accountsType === 'devnet' ? devnet.url : customNetwork

  const fetchDevnetStatus = useCallback(() => {
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
  }, [setIsDevnetAlive, devnet.url])

  useInterval(fetchDevnetStatus, accountsType === 'devnet' && devnet.url.length > 0 ? DEVNET_POLL_INTERVAL : null)

  useEffect(() => {
    if (accountsType === 'devnet') {
      fetchDevnetStatus()
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
    }
  }, [isDevnetAlive, remixClient, devnet, fetchDevnetStatus, accountsType])

  useAsync(async () => {
    const updatedAccounts = await updateBalances(availableDevnetAccounts, networkUrl)
    setAvailableDevnetAccounts(updatedAccounts)
  }, [networkUrl, env])

  const [, refreshDevnetAccounts] = useAsyncFn(async () => {
    try {
      setAccountRefreshing(true)
      const accounts = await getAccounts(networkUrl)
      if (JSON.stringify(accounts) !== JSON.stringify(availableDevnetAccounts)) {
        setAvailableDevnetAccounts(accounts)
      }
    } catch (error) {
      await remixClient.terminal.log({
        type: 'error',
        value: `Failed to get accounts information from ${networkUrl}`
      })
    }
    setAccountRefreshing(false)
  }, [remixClient, networkUrl])

  useEffect(() => {
    refreshDevnetAccounts()
  }, [refreshDevnetAccounts])

  useEffect(() => {
    if (
      !(selectedDevnetAccount !== null && availableDevnetAccounts.includes(selectedDevnetAccount)) &&
      availableDevnetAccounts.length > 0 &&
      isAlive
    ) {
      setSelectedDevnetAccount(availableDevnetAccounts[0])
    }
    if (!isAlive && selectedDevnetAccount !== null) {
      setSelectedDevnetAccount(null)
    }
  }, [availableDevnetAccounts, devnet, selectedDevnetAccount, setSelectedDevnetAccount, env, isAlive])

  useEffect(() => {
    const newProvider = new Provider(networkUrl)
    if (selectedDevnetAccount != null) {
      setAccount(new Wallet(selectedDevnetAccount.private_key, newProvider))
    }
    setProvider(newProvider)
  }, [networkUrl, selectedDevnetAccount, setAccount, setProvider])

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
      <label className="">{accountsType === 'devnet' && 'Devnet'}Account selection</label>
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
                        <div className="devnet-account-item">
                          <span className="selected-account">{accountIdx === index && <BsCheck size={18} />}</span>
                          <span className="ml-2 chain-account-info">
                            <span>{getShortenedHash(account.address ?? '', 6, 4)}</span>
                            <span className="account-balance">
                              {`(${getRoundedNumber(weiToEth(account.initial_balance), 2)} ETH)`}
                            </span>
                          </span>
                        </div>
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
