import React, { useEffect, useState } from 'react'
import { formatEther } from 'ethers/lib/utils'
import { Provider, Wallet } from 'zksync-ethers'
import { useAtom, useSetAtom } from 'jotai'
import copy from 'copy-to-clipboard'
import { FaCheck } from 'react-icons/fa'
import { MdCopyAll } from 'react-icons/md'
import { BsChevronDown } from 'react-icons/bs'
import { CiSquareCheck, CiSquarePlus } from 'react-icons/ci'
import { accountsAtom, selectedAccountAtom } from '@/atoms/manualAccount'

import * as Dropdown from '@/ui_components/Dropdown'
import { getShortenedHash } from '@/utils/utils'
import { accountAtom } from '@/atoms/connection'
import useInterval from '@/hooks/useInterval'
import { ZKSYNC_SEPOLIA_FAUCET_URL, ZKSYNC_SEPOLIA_RPC_URL } from '@/utils/network'
import './index.css'

const provider = new Provider(ZKSYNC_SEPOLIA_RPC_URL)

export const ManualAccount = () => {
  const [dropdownControl, setDropdownControl] = useState(false)
  const setAccount = useSetAtom(accountAtom)

  const [selectedAccount, setSelectedAccount] = useAtom(selectedAccountAtom)
  const [isClicked, setIsClicked] = useState(false)
  const [accounts, setAccounts] = useAtom(accountsAtom)
  const [copied, setCopied] = React.useState(false)

  const copyAddress = (): void => {
    copy(selectedAccount?.address ?? '')
    setCopied(true)
    setTimeout(() => {
      setCopied(false)
    }, 1000)
  }

  const addAccount = (): void => {
    setIsClicked(true)
    setTimeout(() => {
      setIsClicked(false)
    }, 1000)

    const wallet = Wallet.createRandom()
    const newAccount = {
      address: wallet.address,
      private_key: wallet.privateKey,
      public_key: wallet.publicKey,
      balance: '0'
    }

    setSelectedAccount(newAccount)
    setAccounts((prevAccounts) => [newAccount, ...prevAccounts])
  }

  useInterval(
    async () => {
      try {
        if (selectedAccount) {
          const balance = await provider.getBalance(selectedAccount.address)
          setSelectedAccount((prevAccount) => {
            if (prevAccount != null && balance.toString() !== prevAccount.balance) {
              return { ...prevAccount, balance: balance.toString() }
            }
            return prevAccount
          })
        }
      } catch (error) {
        console.error(error)
      }
    },
    selectedAccount !== null ? 10_000 : null
  )

  useEffect(() => {
    if (selectedAccount !== null) {
      const wallet = new Wallet(selectedAccount.private_key, provider)

      setAccount(wallet)
    }
  }, [selectedAccount, setAccount])

  return (
    <div className="manual-root-wrapper">
      <div className={'flex flex-column'}>
        <div className={'flex flex-row justify-content-space-between'}>
          <Dropdown.Root
            open={dropdownControl}
            onOpenChange={(e) => {
              setDropdownControl(e)
            }}
          >
            <Dropdown.Trigger>
              <div className="flex flex-row justify-content-space-between align-items-center p-2 pb-1 br-1 compiled-contracts-wrapper">
                <label>
                  {selectedAccount !== null ? getShortenedHash(selectedAccount.address, 12, 4) : 'No Accounts'}
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
                {accounts.map((account, index) => {
                  return (
                    <Dropdown.Item
                      onClick={() => {
                        setSelectedAccount(account)
                        setDropdownControl(false)
                      }}
                      key={index}
                    >
                      {getShortenedHash(account.address, 12, 4)}
                    </Dropdown.Item>
                  )
                })}
              </Dropdown.Content>
            </Dropdown.Portal>
          </Dropdown.Root>
          <button className={'add-account-button-plus ml-2'} onClick={addAccount}>
            {!isClicked ? <CiSquarePlus /> : <CiSquareCheck />}
          </button>
          <button className="btn" onClick={copyAddress}>
            {copied ? <FaCheck /> : <MdCopyAll />}
          </button>
        </div>
        <div className={'flex flex-row w-100 mb-0 mt-1'}>
          <label>Account Balance: {formatEther(selectedAccount?.balance ?? '0').toString()} ETH</label>
        </div>
        <div className={'flex flex-row w-100 mb-0 mt-2'}>
          <button
            className="btn btn-primary btn-sm btn-block w-100 mb-0 btn-warning"
            onClick={() => {
              copyAddress()
              window.open(ZKSYNC_SEPOLIA_FAUCET_URL)
            }}
          >
            Get Testnet Funds
          </button>
        </div>
      </div>
    </div>
  )
}

export default ManualAccount
