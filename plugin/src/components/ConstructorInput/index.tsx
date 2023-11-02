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

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface ConstructorContractsProps {
  inputs: string[]
  setInputs: (input: string[]) => void
}

const ConstructorInput: React.FC<ConstructorContractsProps> = ({ inputs, setInputs }: ConstructorContractsProps) => {
  const { selectedContract} = useContext(
    CompiledContractsContext
  )

  const [constructor, setConstructor] = useState<AbiElement | undefined>(undefined)

  useEffect(() => {
    const foundConstructor = selectedContract?.abi.find((abiElement: AbiElement) => {
      return abiElement.type === 'constructor'
    })

    setConstructor(foundConstructor)
  }, [selectedContract])

  return (
    <>
      {
        constructor && (constructor as AbiElement).inputs.map((input: Input, index: number) => {
          return (
            <InputField name={input.name} index={index} value={inputs[index]} onChange={(index, newValue) => {
              const newInputs = [...inputs]
              newInputs[index] = newValue
              setInputs(newInputs)
            }}/>
          )
        }
        )
      }
    </>
  )
}

export default ConstructorInput
