// A component that reads the compiled contracts from the context and displays them in a select

import React, { useEffect, useState } from 'react'
import * as zksync from 'zksync-ethers'
import { generateInputName, parseContractInputs } from '../../utils/utils'
import { type AbiElement, type Input } from '../../types/contracts'
import InputField from '../InputField'
import { ethers } from 'ethers'
import { mockManualChain, type Transaction } from '../../types/transaction'
import { useAtom, useAtomValue } from 'jotai'
import { deployedSelectedContractAtom } from '../../atoms/deployedContracts'
import { transactionsAtom } from '../../atoms/transaction'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { useWalletClient } from 'wagmi'
import { envAtom } from '../../atoms/environment'
import { remixClientAtom } from '../../stores/remixClient'
import { ContractInputType } from '../ConstructorInput'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  element: AbiElement
}

const MethodInput: React.FC<CompiledContractsProps> = ({ element }: CompiledContractsProps) => {
  const remixClient = useAtomValue(remixClientAtom)
  const { data: walletClient } = useWalletClient()

  const selectedContract = useAtomValue(deployedSelectedContractAtom)
  const [transactions, setTransactions] = useAtom(transactionsAtom)
  const account = useAtomValue(accountAtom)

  const env = useAtomValue(envAtom)
  const provider = useAtomValue(providerAtom)

  const [inputs, setInputs] = useState<ContractInputType>(
    new Array(element.inputs.length).fill({ internalType: 'string', value: '' })
  )
  const [value, setValue] = useState<string>('')

  const callContract = async (): Promise<void> => {
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
      const contract = new zksync.Contract(contractAddress, selectedContract.abi, account)

      const method = contract[element.name]

      remixClient.emit('statusChanged', {
        key: 'loading',
        type: 'info',
        title: `Executing "${element.name}" method!`
      })

      const callParameters = parseContractInputs(inputs)
      if (element.stateMutability === 'payable') {
        const options: any = { value: ethers.utils.parseEther(value) }
        callParameters.push(options)
      }
      const result = await method(...callParameters)

      remixClient.emit('statusChanged', {
        key: 'succeed',
        type: 'success',
        title: `Executed "${element.name}" method!`
      })

      if (element.stateMutability !== 'view') {
        const transaction: Transaction = {
          account,
          type: 'invoke',
          txId: result.hash,
          env,
          chain: env !== 'manual' ? walletClient?.chain : mockManualChain,
          provider,
          value
        }

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

      if (e instanceof Error) {
        await remixClient.terminal.log({
          value: `Error in function invocation (message): ${JSON.stringify(e.message)}`,
          type: 'error'
        })
      } else {
        await remixClient.terminal.log({
          value: `Error in function invocation: ${JSON.stringify(e)}`,
          type: 'error'
        })
      }

      await remixClient.call('notification' as any, 'toast', `Error: ${String(e)}`)
    }
  }

  useEffect(() => {
    setInputs(new Array(element.inputs.length).fill({ internalType: 'string', value: '' }))
  }, [element])

  return (
    <div>
      {element.inputs.length == inputs.length ? (
        <>
          <button
            onClick={() => {
              callContract().catch(console.error)
            }}
            className={`
          btn btn-primary w-100 text-break mb-1 mt-1 px-0
          ${element.stateMutability === 'view' ? '' : 'btn-warning'}
        `}
          >
            {element.name}
          </button>
          {element.stateMutability === 'payable' ? (
            <InputField
              key={'value'}
              placeholder={'Amount (ETH)'}
              index={element.inputs.length}
              value={value}
              onChange={(_, newValue) => {
                setValue(newValue)
              }}
            />
          ) : (
            <></>
          )}
          {element.inputs.map((input: Input, index: number) => (
            <InputField
              key={index}
              placeholder={generateInputName(input)}
              index={index}
              value={inputs[index].value}
              onChange={(index, newValue) => {
                const newInputs = [...inputs]
                newInputs[index] = {
                  internalType: input.internalType || 'string',
                  value: newValue
                }
                setInputs(newInputs)
              }}
            />
          ))}
        </>
      ) : <></>}
    </div>
  )
}

export default MethodInput
