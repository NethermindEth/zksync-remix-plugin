import React from 'react'
import { RxDotFilled } from 'react-icons/rx'
import { useAtomValue } from 'jotai'
import { envAtom, isCustomNetworkAliveAtom, isDevnetAliveAtom } from '@/atoms'

export const DevnetStatus = () => {
  const env = useAtomValue(envAtom)
  const isDevnetAlive = useAtomValue(isDevnetAliveAtom)
  const isCustomNetworkAlive = useAtomValue(isCustomNetworkAliveAtom)

  if (env === 'wallet') {
    return <RxDotFilled size={'30px'} color="rebeccapurple" title="Wallet is active" />
  }
  if (env === 'customNetwork') {
    return isCustomNetworkAlive ? (
      <RxDotFilled size={'30px'} color="lime" title="Devnet is live" />
    ) : (
      <RxDotFilled size={'30px'} color="red" title="Devnet server down" />
    )
  }
  return isDevnetAlive ? (
    <RxDotFilled size={'30px'} color="lime" title="Devnet is live" />
  ) : (
    <RxDotFilled size={'30px'} color="red" title="Devnet server down" />
  )
}
