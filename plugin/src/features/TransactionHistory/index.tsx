import React from 'react'
import Container from '@/ui_components/Container'
import { TransactionCard } from './TransactionCard'
import { useAtomValue } from 'jotai'
import { transactionsAtom } from '@/atoms'

export const TransactionHistory = () => {
  const transactions = useAtomValue(transactionsAtom)
  return (
    <Container>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.8rem' }}>
        {transactions.length === 0 ? (
          <div className={'w-100 text-center font-bold pt-4'}>No transactions yet</div>
        ) : (
          transactions.map((transaction) => <TransactionCard key={transaction.txId} transaction={transaction} />)
        )}
      </div>
    </Container>
  )
}
