import React, { useEffect, useState } from 'react'

import CompiledContracts from '../../components/CompiledContracts'
import './styles.css'
import Container from '../../ui_components/Container'

import { type AccordianTabs } from '../Plugin'
import * as zksync from 'zksync-web3'
import ConstructorInput from '../../components/ConstructorInput'
import { type DeployedContract } from '../../types/contracts'
import { type Transaction } from '../../types/transaction'
import { type Contract } from 'ethers'
import { useWalletClient } from 'wagmi'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { transactionsAtom } from '../../atoms/transaction'
import useRemixClient from '../../hooks/useRemixClient'
import { contractsAtom, selectedContractAtom } from '../../atoms/compiledContracts'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { deployedContractsAtom, deployedSelectedContractAtom } from '../../atoms/deployedContracts'
import { envAtom } from '../../atoms/environment'

interface DeploymentProps {
  setActiveTab: (tab: AccordianTabs) => void
}

const Deployment: React.FC<DeploymentProps> = ({ setActiveTab }) => {
  const { data: walletClient } = useWalletClient()
  const { remixClient } = useRemixClient()
  const [transactions, setTransactions] = useAtom(transactionsAtom)

  const contracts = useAtomValue(contractsAtom)
  const selectedContract = useAtomValue(selectedContractAtom)

  const account = useAtomValue(accountAtom)

  const [deployedContracts, deployedSetContracts] = useAtom(deployedContractsAtom)
  const deployedSetSelectedContract = useSetAtom(deployedSelectedContractAtom)

  const [inputs, setInputs] = useState<string[]>([])

  const env = useAtomValue(envAtom)
  const provider = useAtomValue(providerAtom)

  useEffect(() => {
    const constructor = selectedContract?.abi.find((abiElement) => {
      return abiElement.type === 'constructor'
    })

    if (constructor === undefined || constructor?.inputs === undefined) {
      setInputs([])
      return
    }

    setInputs(new Array(constructor?.inputs.length).fill(''))
  }, [selectedContract])

  async function deploy (): Promise<void> {
    //   Deploy contract
    if (selectedContract == null) {
      await remixClient.call(
        'notification' as any,
        'toast',
        'No contract selected'
      )

      return
    }

    if (account == null) {
      await remixClient.call(
        'notification' as any,
        'toast',
        'No account selected'
      )

      return
    }

    await remixClient.terminal.log({
      value: `Deploying contract ${selectedContract.contractName}`,
      type: 'info'
    })

    const factory = new zksync.ContractFactory(
      selectedContract.abi,
      selectedContract.bytecode,
      account
    )

    try {
      const contract: Contract = await factory.deploy(...inputs)

      remixClient.emit('statusChanged', {
        key: 'loading',
        type: 'info',
        title: `Contract ${selectedContract.contractName} is deploying!`
      })

      const tx = await contract.deployed()

      remixClient.emit('statusChanged', {
        key: 'succeed',
        type: 'success',
        title: `Contract ${selectedContract.contractName} deployed!`
      })

      const address = tx.address
      const txHash = tx.deployTransaction.hash

      const contractOutputTx = tx.deployTransaction

      contractOutputTx.data = contractOutputTx.data.slice(0, contractOutputTx.data.length / 3) + '...'

      // @ts-expect-error: customData is returned properly but the type is not defined
      contractOutputTx.customData.factoryDeps = '[ <...> ]'

      await remixClient.terminal.log({
        value: `${JSON.stringify(contractOutputTx, null, 2)}`,
        type: 'info'
      })

      const deployedContract: DeployedContract = {
        ...selectedContract,
        bytecode: selectedContract.bytecode,
        transactionHash: txHash,
        address
      }

      deployedSetContracts([deployedContract, ...deployedContracts])
      deployedSetSelectedContract(deployedContract)

      setActiveTab('interaction')

      const transaction: Transaction = {
        account,
        type: 'deploy',
        txId: txHash,
        env,
        chain: walletClient?.chain,
        provider
      }

      setTransactions([transaction, ...transactions])
    } catch (e) {
      await remixClient.terminal.log({
        value: `Error: ${JSON.stringify(e)}`,
        type: 'error'
      })

      remixClient.emit('statusChanged', {
        key: 'failed',
        type: 'error',
        title: `Contract ${selectedContract.contractName} failed to deploy!`
      })

      await remixClient.call(
        'notification' as any,
        'toast',
        `Error: ${JSON.stringify(e)}`
      )
    }
  }

  return (
    <>
      <Container>
        {contracts.length > 0
          ? (
            <div>
              <CompiledContracts show={'contract'}></CompiledContracts>
              {
                (selectedContract != null)
                  ? <div>
                    <ConstructorInput inputs={inputs} setInputs={setInputs}></ConstructorInput>

                    <button
                      className='deploy-btn btn btn-primary btn-warning w-100 text-break mb-1 mt-2 px-0'
                      onClick={() => {
                        deploy().catch((err) => {
                          console.log(err)
                        })
                      }}
                    >
                      Deploy
                    </button>

                  </div>
                  : <>
                  </>
              }
            </div>
            )
          : (
            <p>No contracts ready for deployment yet, compile a solidity contract</p>
            )}
      </Container>
    </>
  )
}

export default Deployment
