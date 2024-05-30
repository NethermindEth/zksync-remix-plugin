import React from 'react'
import { useAtomValue } from 'jotai'
import Container from '@/ui_components/Container'
import './index.css'
import DeployedContracts from '@/components/DeployedContracts'
import { deployedContractsAtom, deployedSelectedContractAtom } from '@/atoms/deployedContracts'

export const Interaction = () => {
  const contracts = useAtomValue(deployedContractsAtom)
  const selectedContract = useAtomValue(deployedSelectedContractAtom)

  return (
    <Container>
      {contracts.length > 0 && selectedContract != null ? (
        <DeployedContracts />
      ) : (
        <div>
          <p>No deployed contracts to interact with... Yet.</p>
        </div>
      )}
    </Container>
  )
}
