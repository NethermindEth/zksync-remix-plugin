/* eslint-disable @typescript-eslint/no-misused-promises */
import React, { useEffect, useState } from 'react'
import { networks as networkConstants } from '../../utils/constants'
import { ethers } from 'ethers'

import storage from '../../utils/storage'

import './index.css'
import { BiCopy, BiPlus } from 'react-icons/bi'
import { trimStr } from '../../utils/utils'
import { MdCheckCircleOutline, MdRefresh } from 'react-icons/md'
import copy from 'copy-to-clipboard'
import useRemixClient from '../../hooks/useRemixClient'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { envAtom } from '../../atoms/environment'
import { accountsAtom, networkNameAtom, selectedAccountAtom } from '../../atoms/manualAccount'
import { useAtomValue, useSetAtom, useAtom } from 'jotai'
import { type EnvType } from '../../types/transaction'

// TODOS: move state parts to contexts
// Account address selection
// network selection drop down
const ManualAccount: React.FC<{
  prevEnv: EnvType
}> = ({ prevEnv }) => {
  const { remixClient } = useRemixClient()

  const account = useAtomValue(accountAtom)
  const provider = useAtomValue(providerAtom)

  const setEnv = useSetAtom(envAtom)

  const [accounts, setAccounts] = useAtom(accountsAtom)
  const selectedAccount = useAtomValue(selectedAccountAtom)
  const [networkName, setNetworkName] = useAtom(networkNameAtom)

  const [accountDeploying] = useState(false)

  useEffect(() => {
    setNetworkName(networkConstants[0].value)
  }, [setNetworkName])

  useEffect(() => {
    const manualAccounts = storage.get('manualAccounts')
    if (
      manualAccounts != null &&
      accounts.length === 0 &&
      selectedAccount == null
    ) {
      const parsedAccounts = JSON.parse(manualAccounts)
      setAccounts(parsedAccounts)
    }
  })

  return (
    <div className='manual-root-wrapper'>
      <button
        type='button'
        className='mb-0 btn btn-sm btn-outline-secondary float-right rounded-pill env-testnet-btn'
        onClick={() => {
          setEnv(prevEnv)
        }}
      >
        Back to Previous
      </button>
      <div className='network-selection-wrapper'>
        <select
          className='custom-select'
          aria-label='.form-select-sm example'
          defaultValue={
            selectedAccount == null
              ? -1
              : accounts.findIndex(
                (acc) => acc.address === selectedAccount?.address
              )
          }
        >
          {accounts.length > 0
            ? (
                accounts.map((account, index) => {
                  return (
                  <option value={index} key={index}>
                    {trimStr(account.address, 6)}
                  </option>
                  )
                })
              )
            : (
              <option value={-1} key={-1}>
                No account created yet
              </option>
              )}
        </select>
        <button
          className='btn btn-primary'
          onClick={(e) => {
            e.preventDefault()
          }}
        >
          <BiPlus />
        </button>
      </div>
      {selectedAccount != null && (
        <div>
          <div className='mb-2'>
            <div className='selected-address-wrapper'>
              {account != null && (
                <p className='m-0'>
                  Address:{' '}
                  <a
                    target='_blank'
                    rel='noreferer noopener noreferrer'
                  >
                    {trimStr(selectedAccount.address, 8)}
                  </a>
                </p>
              )}
              <div className='d-flex'>
                <button
                  className='btn'
                  onClick={() => copy(selectedAccount.address)}
                >
                  <BiCopy />
                </button>
              </div>
            </div>
          </div>
          {account != null && provider != null && (
            <div className='manual-balance-wrapper'>
              <p>
                Balance:{' '}
                {parseFloat(
                  ethers.utils.formatEther(selectedAccount.balance)
                )?.toFixed(8)}{' '}
                ETH
              </p>
              <button
                className='btn btn-refresh'
                onClick={(e) => {
                  e.preventDefault()
                }}
              >
                <MdRefresh />
              </button>
            </div>
          )}
          {networkName === 'goerli-alpha' && (
            <button
              className='btn btn-secondary w-100'
              onClick={() => {
                copy(selectedAccount?.address ?? '')
                remixClient.call('notification' as any, 'toast', 'ℹ️ Address copied to Clipboard').catch((err) => {
                  console.log(err)
                })
                setTimeout(() => {
                  window?.open(
                    'https://faucet.goerli.zksync.io/',
                    '_blank',
                    'noopener noreferrer'
                  )
                }, 2000)
              }}
            >
              Request funds on Zksync Faucet
            </button>
          )}
        </div>
      )}

      <select
        className='custom-select'
        aria-label='.form-select-sm example'
        value={networkName}
        defaultValue={networkName}
      >
        {networkConstants.map((network) => {
          return (
            <option value={network.value} key={network.name}>
              {network.value}
            </option>
          )
        })}
      </select>
      <button
        className='btn btn-primary btn-block d-block w-100 text-break remixui_disabled'
        style={{
          cursor: `${
            (selectedAccount?.deployed_networks.includes(networkName) ??
              false) ||
            accountDeploying
              ? 'not-allowed'
              : 'pointer'
          }`
        }}
        disabled={
          (selectedAccount?.deployed_networks.includes(networkName) ?? false) ||
          accountDeploying
        }
        aria-disabled={
          (selectedAccount?.deployed_networks.includes(networkName) ?? false) ||
          accountDeploying
        }
        onClick={(e) => {
          e.preventDefault()
        }}
      >
        {accountDeploying
          ? (
            <>
            <span
              className='spinner-border spinner-border-sm'
              role='status'
              aria-hidden='true'
            />
              <span style={{ paddingLeft: '0.5rem' }}>Deploying Account...</span>
            </>
            )
          : selectedAccount?.deployed_networks.includes(networkName) ??
          false
            ? (
              <>
                <MdCheckCircleOutline color='#0fd543' size={18} />
                <span style={{ paddingLeft: '0.5rem' }}>Account Deployed</span>
              </>
              )
            : (
                'Deploy Account'
              )}
      </button>
    </div>
  )
}

export default ManualAccount
