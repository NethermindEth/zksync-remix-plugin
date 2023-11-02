// A component that reads the compiled contracts from the context and displays them in a select

import React, { useContext, useEffect, useState } from 'react'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import {
  getContractNameFromFullName,
  getSelectedContractIndex,
  getShortenedHash
} from '../../utils/utils'
import { type AbiElement, type Input } from '../../types/contracts'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import InputField from '../InputField'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'
import { Contract } from 'ethers'
import { Provider, Wallet } from 'zksync-web3'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface CompiledContractsProps {
  element: AbiElement
}

const MethodInput: React.FC<CompiledContractsProps> = ({ element }: CompiledContractsProps) => {
  const [inputs, setInputs] = useState<string[]>([])
  const { selectedContract } = useContext(DeployedContractsContext)
  const remixClient = useContext(RemixClientContext)

  const callContract = async () => {
    if (selectedContract == null) {
      remixClient.terminal.log('No contract selected' as any)
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
      const result = await method(...inputs)

      remixClient.terminal.log(`${JSON.stringify(result, null, 2)}` as any)
    } catch (e) {
      remixClient.terminal.log(`Error: ${e}` as any)
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
              <InputField placeholder={input.name} index={index} value={inputs[index]} onChange={(index, newValue) => {
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
