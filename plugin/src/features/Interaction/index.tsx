/* eslint-disable react/jsx-key */
/* eslint-disable @typescript-eslint/strict-boolean-expressions */
/* eslint-disable @typescript-eslint/explicit-function-return-type */
/* eslint-disable no-case-declarations */
/* eslint-disable multiline-ternary */
import React, { useContext } from 'react'

import Container from '../../ui_components/Container'
import './index.css'
import DeployedContracts from '../../components/DeployedContracts'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface InteractionProps {
  setInteractionStatus: React.Dispatch<React.SetStateAction<'loading' | 'error' | 'success' | ''>>
}

const Interaction: React.FC<InteractionProps> = (props) => {
  const {
    contracts,
    selectedContract
  } = useContext(DeployedContractsContext)

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

export default Interaction
