/* eslint-disable @typescript-eslint/no-misused-promises */
import React, { useEffect, useRef, useState } from 'react'

import './index.css'
import { envAtom } from '../../atoms/environment'
import { accountsAtom, selectedAccountAtom } from '../../atoms/manualAccount'
import { useAtom, useSetAtom } from 'jotai'
import { type EnvType } from '../../types/transaction'
import * as D from '../../ui_components/Dropdown'
import { BsChevronDown } from 'react-icons/bs'
import { CiSquareCheck, CiSquarePlus } from 'react-icons/ci'
import { Provider, Wallet } from 'zksync-ethers'
import { getShortenedHash } from '../../utils/utils'
import copy from 'copy-to-clipboard'
import { FaCheck } from 'react-icons/fa'
import { MdCopyAll } from 'react-icons/md'
import { formatEther } from 'ethers/lib/utils'
import { accountAtom } from '../../atoms/connection'

// TODOS: move state parts to contexts
// Account address selection
// network selection drop down
const ManualAccountComp: React.FC<{
  prevEnv: EnvType
}> = ({ prevEnv }) => {
  const setEnv = useSetAtom(envAtom)

  const [dropdownControl, setDropdownControl] = useState(false)

  const setAccount = useSetAtom(accountAtom)

  const [selectedAccount, setSelectedAccount] = useAtom(selectedAccountAtom)
  const [isClicked, setIsClicked] = useState(false)
  const [accounts, setAccounts] = useAtom(accountsAtom)
  const balanceUpdateIntervalRef = useRef<null | NodeJS.Timeout>(null)
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
    setAccounts(prevAccounts => [newAccount, ...prevAccounts])
  }

  const updateBalance = async (): Promise<void> => {
    if (selectedAccount != null) {
      const provider = new Provider('https://testnet.era.zksync.dev')
      const balance = await provider.getBalance(selectedAccount.address)
      setSelectedAccount(prevAccount => {
        if ((prevAccount != null) && balance.toString() !== prevAccount.balance) {
          return { ...prevAccount, balance: balance.toString() }
        }
        return prevAccount
      })
    }
  }

  useEffect(() => {
    if (balanceUpdateIntervalRef.current !== null) {
      clearInterval(balanceUpdateIntervalRef.current)
    }

    if (selectedAccount !== null) {
      balanceUpdateIntervalRef.current = setInterval(updateBalance, 1000)
    }

    return () => {
      if (balanceUpdateIntervalRef.current !== null) {
        clearInterval(balanceUpdateIntervalRef.current)
      }
    }
  }, [selectedAccount])

  useEffect(() => {
    if (selectedAccount !== null) {
      const provider = new Provider('https://testnet.era.zksync.dev')
      const wallet = new Wallet(selectedAccount.private_key, provider)

      setAccount(wallet)
    }
  }, [selectedAccount])

  return (
    <div className='manual-root-wrapper'>
      <button
        type='button'
        className='mb-0 btn btn-sm btn-primary float-right rounded-pill'
        onClick={() => {
          setEnv(prevEnv)
        }}
      >
        Back to Previous
      </button>

      <div className={'flex flex-column'}>
        <div className={'flex flex-row justify-content-space-between'}>
          <D.Root open={dropdownControl} onOpenChange={(e) => { setDropdownControl(e) }}>
            <D.Trigger>
              <div className='flex flex-row justify-content-space-between align-items-center p-2 pb-1 br-1 compiled-contracts-wrapper'>
                <label>
                  {selectedAccount !== null ? getShortenedHash(selectedAccount.address, 16, 4) : 'No Accounts'}
                </label>
                <BsChevronDown style={{
                  transform: dropdownControl ? 'rotate(180deg)' : 'none',
                  transition: 'all 0.3s ease'
                }} />
              </div>
            </D.Trigger>
            <D.Portal>
              <D.Content>
                {accounts.map((account, index) => {
                  return (
                    <D.Item
                      onClick={() => {
                        setSelectedAccount(account)
                        setDropdownControl(false)
                      }}
                      key={index}
                    >
                      {getShortenedHash(account.address, 20, 4)}
                    </D.Item>
                  )
                })}
              </D.Content>
            </D.Portal>
          </D.Root>
          <button className={'add-account-button-plus ml-2'} onClick={addAccount}>
            { !isClicked ? <CiSquarePlus /> : <CiSquareCheck /> }
          </button>
          <button
            className='btn'
            onClick={copyAddress}
          >
            {copied ? <FaCheck /> : <MdCopyAll />}
          </button>
        </div>
        <div className={'flex flex-row w-100 mb-0 mt-1'}>
          <label>Account Balance: {formatEther(selectedAccount?.balance ?? '0').toString()} ETH</label>
        </div>
        <div className={'flex flex-row w-100 mb-0 mt-2'}>
          <button className='btn btn-primary btn-sm btn-block w-100 mb-0 btn-warning' onClick={() => {
            copyAddress()
            window.open('https://faucet.triangleplatform.com/zksync/testnet')
          }}>
            Get Testnet Funds
          </button>
        </div>
      </div>
    </div>
  )
}

export default ManualAccountComp
