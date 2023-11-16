import React from 'react'
import Container from '../../ui_components/Container'
import TransactionCard from './TransactionCard'
import { useAtomValue } from 'jotai'
import { transactionsAtom } from '../../atoms/transaction'

const TransactionHistory: React.FC = () => {
  const transactions = useAtomValue(transactionsAtom)

  return (
    <Container>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '0.8rem'
      }}>
        {transactions.length === 0
          ? (
            <div>No transactions yet</div>
            )
          : (
              transactions.map((transaction, index) => {
                return <TransactionCard key={transaction.txId} transaction={transaction}
              />
              })
            )
        }
      </div>
    </Container>
  )
}

export default TransactionHistory
