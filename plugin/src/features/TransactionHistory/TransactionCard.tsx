import React, { useEffect, useMemo, useRef, useState } from 'react'
import { type Transaction } from '../../types/transaction'
import { getShortenedHash } from '../../utils/utils'
import './transactioncard.css'
interface TagType {
  type: 'deploy' | 'declare' | 'invoke' | 'deployAccount'
}

const Tag = ({ type }: TagType) => {
  return <span className={`p-2 tag tag-${type}`}>{type === 'deployAccount' ? 'deploy account' : type}</span>
}

const transformTypeToText = (type: string): string => {
  switch (type) {
    case 'localDevnet':
      return 'Local Devnet'
    case 'remoteDevnet':
      return 'Remote Devnet'
    default:
      return type
  }
}

const NetworkTag = ({ type }: { type: string }) => {
  return <span className={`p-2 tag tag-${type}`}>{transformTypeToText(type)}</span>
}

export const TransactionCard = ({ transaction }: { transaction: Transaction }) => {
  const cardRef = useRef<HTMLDivElement>(null)
  const [address, setAddress] = useState<string | undefined>()
  const { account, txId: txnHash, env, chain } = transaction
  const isEnvLocalOrRemote = env === 'localDevnet' || env === 'remoteDevnet'
  const txnHashShort = getShortenedHash(txnHash, 8, 6)

  useEffect(() => {
    const fetchAddress = async (): Promise<void> => {
      const addr = await account?.getAddress()
      setAddress(addr)
    }

    fetchAddress().catch(console.error)
  }, [account])

  const accountShort = useMemo(() => getShortenedHash(address || '', 8, 6), [address])

  return (
    <div className="maincard" ref={cardRef}>
      <div className={'txn-info-1'}>
        <div className="account-wrapper">
          <p className={'label-tx'}>From:</p>
          {isEnvLocalOrRemote ? (
            <a title={address} target="_blank" rel="noreferrer">
              {accountShort}
            </a>
          ) : (
            <a
              title={address}
              href={`${String(chain?.blockExplorers?.default.url)}/address/${address ?? ''}`}
              target="_blank"
              rel="noreferrer"
            >
              {accountShort}
            </a>
          )}
        </div>
        <div className="txn-wrapper">
          <p className={'label-tx'}>TxHash:</p>
          {isEnvLocalOrRemote ? (
            <a target="_blank" title={txnHash} rel="noreferrer">
              {txnHashShort}
            </a>
          ) : (
            <a
              href={`${String(chain?.blockExplorers?.default.url)}/tx/${txnHash}`}
              target="_blank"
              title={txnHash}
              rel="noreferrer"
            >
              {txnHashShort}
            </a>
          )}
        </div>
      </div>
      <div className={'txn-info-2'}>
        <div className="tag-wrapper">
          <Tag type={transaction.type} />
        </div>
        <div className="txn-network">
          <NetworkTag type={isEnvLocalOrRemote ? env : chain?.name === undefined ? '' : chain.name} />
        </div>
      </div>
    </div>
  )
}
