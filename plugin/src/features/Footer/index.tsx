import React from 'react'
import Nethermind from '../../components/NM'
import './style.css'
import { useAtomValue } from 'jotai'
import { solidityVersionAtom } from '@/atoms'

export const Footer = () => {
  const solidityVersion = useAtomValue(solidityVersionAtom)

  return (
    <div className="version-wrapper">
      <div>
        <label className="version-left">
          <span>Using</span>
          <span>zksolc-{solidityVersion}</span>
        </label>
      </div>
      <div className="version-right">
        <label className="nethermind-powered">
          <span>Powered by: </span>
          <Nethermind size="xs" />
        </label>
      </div>
    </div>
  )
}
