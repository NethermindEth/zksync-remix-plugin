import React from 'react'
import { useAtomValue } from 'jotai'
import { selectedAccountAtom, accountInfoAtom, envAtom, selectedDevnetAccountAtom } from '@/atoms'
import { getShortenedHash, weiToEth } from '@/utils/utils'
import { envName } from '@/utils/misc'
import { DevnetStatus } from '@/components'
import './currentEnv.css'

export const CurrentEnv = () => {
  const env = useAtomValue(envAtom)

  const selectedAccountManual = useAtomValue(selectedAccountAtom)
  const selectedAccountDevnet = useAtomValue(selectedDevnetAccountAtom)
  const walletAccountInfo = useAtomValue(accountInfoAtom)

  const selectedAccount =
    env === 'wallet'
      ? walletAccountInfo
      : env === 'manual'
        ? { address: selectedAccountManual?.address, balance: selectedAccountManual?.balance }
        : { address: selectedAccountDevnet?.address, balance: selectedAccountDevnet?.initial_balance }

  const selectedAccountAddress =
    selectedAccount.address != null ? getShortenedHash(selectedAccount.address, 6, 4) : 'No account selected'

  const selectedAccountBalance = weiToEth(Number(selectedAccount.balance ?? 0))

  return (
    <div className={'current-env-root'}>
      <div className={'devnet-status'}>
        <DevnetStatus />
      </div>
      <div className={'chain-info-box'}>
        <span className={'chain-name'}>{envName(env)}</span>
        <span className={'chain-account-info'}>
          <span>{selectedAccountAddress}</span>
          <span className="account-balance">{selectedAccount != null ? `(${selectedAccountBalance} ETH)` : ''}</span>
        </span>
      </div>
    </div>
  )
}
