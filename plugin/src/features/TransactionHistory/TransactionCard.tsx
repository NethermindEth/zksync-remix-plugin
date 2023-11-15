import React, { useEffect, useRef, useState } from 'react'
import { type Transaction } from '../../types/transaction'
import './transactioncard.css'

interface TagType {
  type: 'deploy' | 'declare' | 'invoke' | 'deployAccount'
}

const Tag: React.FC<TagType> = ({ type }) => {
  return <span className={`p-2 tag tag-${type}`}>{type === 'deployAccount' ? 'deploy account' : type}</span>
}

interface NetworkTypeTag {
  type: string
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

const NetworkTag: React.FC<NetworkTypeTag> = ({ type }) => {
  return (
    <span className={`p-2 tag tag-${type}`}>{transformTypeToText(type)}</span>
  )
}

interface TransactionCardProps {
  transaction: Transaction
  // explorer: keyof typeof networkExplorerUrls
}

const TransactionCard: React.FC<TransactionCardProps> = ({
  transaction
}) => {
  const {
    account,
    txId,
    env
    , chain
  } = transaction
  const [address, setAddress] = useState<string | undefined>(undefined)

  useEffect(() => {
    const fetchAddress = async (): Promise<void> => {
      const addr = await account?.getAddress()
      setAddress(addr)
    }

    fetchAddress().catch(console.error)
  }, [account])

  const cardRef = useRef<HTMLDivElement>(null)

  return (
    <div className='maincard' ref={cardRef}>
      <div className='tag-wrapper'>
        <Tag type={transaction.type} />
      </div>
      <div className='account-wrapper'>
        <p>From: </p>
        {(env === 'localDevnet' || env === 'remoteDevnet')
          ? <a
            title={address}
            target='_blank' rel='noreferrer'
          >
            {address}
          </a>
          : <a
            title={address}
            href={`${String(chain?.blockExplorers?.default.url)}/address/${address ?? ''}`}
            target='_blank' rel='noreferrer'
          >
            {address}
          </a>}
      </div>
      <div className='txn-wrapper'>
        <p>Transaction ID</p>
        {(env === 'localDevnet' || env === 'remoteDevnet')
          ? <a target='_blank' title={txId} rel='noreferrer'>
            {txId}
          </a>
          : <a href={`${String(chain?.blockExplorers?.default.url)}/tx/${txId}`} target='_blank' title={txId}
               rel='noreferrer'>
            {txId}
          </a>}
      </div>
      <div className='txn-network'>
        {(env === 'localDevnet' || env === 'remoteDevnet') ? <p>Network</p> : <p>Chain</p>}
        <NetworkTag type={(env === 'localDevnet' || env === 'remoteDevnet') ? env : (chain?.name === undefined ? '' : chain?.name)} />
      </div>
    </div>
  )
}

export default TransactionCard
