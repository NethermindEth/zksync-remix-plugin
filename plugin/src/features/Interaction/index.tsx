/* eslint-disable react/jsx-key */
/* eslint-disable @typescript-eslint/strict-boolean-expressions */
/* eslint-disable @typescript-eslint/explicit-function-return-type */
/* eslint-disable no-case-declarations */
/* eslint-disable multiline-ternary */
import React, { useContext, useEffect, useState } from 'react'
import { type BigNumberish } from 'ethers'

import {
  type Account,
  type RawCalldata,
  type CallContractResponse,
  type GetTransactionReceiptResponse,
  type AccountInterface,
  type InvokeFunctionResponse,
  constants
} from 'starknet'
import CompiledContracts from '../../components/CompiledContracts'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import { type Input } from '../../types/contracts'
import {
  getParameterType,
  getReadFunctions,
  getWriteFunctions
} from '../../utils/utils'
import Container from '../../ui_components/Container'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import TransactionContext from '../../contexts/TransactionContext'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import storage from '../../utils/storage'
import './index.css'
import { useAtom } from 'jotai'
import { type EnhancedAbiElement, interactAtom } from '../../atoms'
import { Formik } from 'formik'
import Yup, { transformInputs } from '../../utils/yup'

import { BiReset } from 'react-icons/bi'
import EnvironmentContext from '../../contexts/EnvironmentContext'
import DeployedContracts from '../../components/DeployedContracts'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface InteractionProps {
  setInteractionStatus: React.Dispatch<React.SetStateAction<'loading' | 'error' | 'success' | ''>>
}

const Interaction: React.FC<InteractionProps> = (props) => {
  const { contracts, selectedContract } = useContext(DeployedContractsContext)

  const remixClient = useContext(RemixClientContext)

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
