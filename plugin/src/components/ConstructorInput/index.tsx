// A component that reads the compiled contracts from the context and displays them in a select

import React, { useEffect, useState } from 'react'
import { generateInputName } from '../../utils/utils'
import { type AbiElement, type Input } from '../../types/contracts'
import InputField from '../InputField'
import { useAtomValue } from 'jotai'
import { selectedContractAtom } from '../../atoms/compiledContracts'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface ConstructorContractsProps {
  inputs: string[]
  setInputs: (input: string[]) => void
}

const ConstructorInput: React.FC<ConstructorContractsProps> = ({
  inputs,
  setInputs
}: ConstructorContractsProps) => {
  const selectedContract = useAtomValue(selectedContractAtom)

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
        constructor?.inputs.map((input: Input, index: number) => {
          return (
              <InputField name={generateInputName(input)} index={index} value={inputs[index]}
                          onChange={(index, newValue) => {
                            const newInputs = [...inputs]
                            newInputs[index] = newValue
                            setInputs(newInputs)
                          }} key={index} />
          )
        }
        )
      }
    </>
  )
}

export default ConstructorInput
