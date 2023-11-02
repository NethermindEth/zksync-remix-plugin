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

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  element: AbiElement
}

const MethodInput: React.FC<CompiledContractsProps> = ({ element }: CompiledContractsProps) => {
  const [inputs, setInputs] = useState<string[]>([])
  const { selectedContract } = useContext(DeployedContractsContext)
  const remixClient = useContext(RemixClientContext)
  const { transactions, setTransactions } = useContext(TransactionContext)

  const callContract = async () => {
    if (selectedContract == null) {
      await remixClient.terminal.log('No contract selected' as any)
      return
    }

    try {
      const contractAddress = selectedContract.address
      const zkSyncProvider = new Provider('http://localhost:8011/')
      const PRIVATE_KEY: string = '0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c'

      const wallet = new Wallet(PRIVATE_KEY, zkSyncProvider)

      const contract = new Contract(contractAddress, selectedContract.abi, wallet)
        .connect(wallet)

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
          type: 'invoke',
          txId: result.hash,
          env: 'localhost'
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
