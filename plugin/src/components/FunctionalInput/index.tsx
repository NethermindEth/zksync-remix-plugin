// A component that reads the compiled contracts from the context and displays them in a select

import React, { useContext, useEffect, useState } from 'react'
import {
  generateInputName
} from '../../utils/utils'
import { type AbiElement, type Input } from '../../types/contracts'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import InputField from '../InputField'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'
import { Contract } from 'ethers'
import { Provider, Wallet } from 'zksync-web3'
import TransactionContext from '../../contexts/TransactionContext'
import { type Transaction } from '../../types/transaction'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import EnvironmentContext from '../../contexts/EnvironmentContext'
import { useWalletClient } from 'wagmi'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  element: AbiElement
}

const MethodInput: React.FC<CompiledContractsProps> = ({ element }: CompiledContractsProps) => {
  const [inputs, setInputs] = useState<string[]>([])
  const { selectedContract } = useContext(DeployedContractsContext)
  const remixClient = useContext(RemixClientContext)
  const { transactions, setTransactions } = useContext(TransactionContext)
  const { account } = useContext(ConnectionContext)
  const { env } = useContext(EnvironmentContext)
  const { data: walletClient } = useWalletClient()

  const callContract = async () => {
    if (selectedContract == null) {
      await remixClient.terminal.log('No contract selected' as any)
      return
    }

    try {
      if (account == null) {
        await remixClient.terminal.log('No account selected' as any)
        return
      }

      const contractAddress = selectedContract.address
      const contract = new Contract(contractAddress, selectedContract.abi, account)
        .connect(account)

      const method = contract[element.name]

      remixClient.emit('statusChanged', {
        key: 'loading',
        type: 'info',
        title: `Executing "${element.name}" method!`
      })

      const result = await method(...inputs)

      remixClient.emit('statusChanged', {
        key: 'succeed',
        type: 'success',
        title: `Executed "${element.name}" method!`
      })

      if (element.stateMutability !== 'view') {
        const transaction = {
          account: account,
          type: 'invoke',
          txId: result.hash,
          env: env,
          chain: walletClient?.chain
        } as Transaction

        setTransactions([transaction, ...transactions])
      }

      if (element.stateMutability === 'view') {
        await remixClient.terminal.log({
          value: `${JSON.stringify(result.toString(), null, 2)}`,
          type: 'info'
        })
      } else {
        await remixClient.terminal.log({
          value: `${JSON.stringify(result, null, 2)}`,
          type: 'info'
        })
      }
    } catch (e) {
      remixClient.emit('statusChanged', {
        key: 'failed',
        type: 'error',
        title: `Contract ${selectedContract.contractName} failed to deploy!`
      })

      await remixClient.call(
        'notification' as any,
        'toast',
        `Error: ${(e as any).code}`
      )
    }
  }

  useEffect(() => {
    setInputs(new Array(element.inputs.length).fill(''))
  }, [element])

  return (
    <>
      <button onClick={() => {
        callContract()
      }} className={`btn btn-primary btn-block d-block w-100 text-break mb-1 mt-2 px-0 ${
        element.stateMutability === 'view' ? '' : 'btn-warning'
      }`} >{element.name}</button>
      {
        element.inputs.map((input: Input, index: number) => {
          return (
            <div>
              <InputField placeholder={generateInputName(input)} index={index} value={inputs[index]} onChange={(index, newValue) => {
                const newInputs = [...inputs]
                newInputs[index] = newValue
                setInputs(newInputs)
              }}/>
            </div>
          )
        })
      }
    </>
  )
}

export default MethodInput
