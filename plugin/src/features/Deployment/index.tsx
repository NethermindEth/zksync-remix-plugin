import React, { useContext, useEffect, useState } from 'react'

import CompiledContracts from '../../components/CompiledContracts'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import './styles.css'
import Container from '../../ui_components/Container'

import { RemixClientContext } from '../../contexts/RemixClientContext'
import { type AccordianTabs } from '../Plugin'
import TransactionContext from '../../contexts/TransactionContext'
import { Wallet, Provider } from 'zksync-web3'
import * as zksync from 'zksync-web3'
import ConstructorInput from '../../components/ConstructorInput'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'
import { type DeployedContract } from '../../types/contracts'
import { type Transaction } from '../../types/transaction'

interface DeploymentProps {
  setActiveTab: (tab: AccordianTabs) => void
}

const Deployment: React.FC<DeploymentProps> = ({ setActiveTab }) => {
  const remixClient = useContext(RemixClientContext)
  const { transactions, setTransactions } = useContext(TransactionContext)
  const { contracts, selectedContract, setContracts, setSelectedContract } =
    useContext(CompiledContractsContext)

  const {
    contracts: deployedContracts, selectedContract: deployedSelectedContract,
    setContracts: deployedSetContracts, setSelectedContract: deployedSetSelectedContract
  } =
      useContext(DeployedContractsContext)

  const [inputs, setInputs] = useState<string[]>([])

  useEffect(() => {
    setInputs(new Array(selectedContract?.abi.find((abiElement) => {
      return abiElement.type === 'constructor'
    })?.inputs.length).fill(''))
  }, [selectedContract])

  async function deploy () {
    // TODO: Get provider
    const zkSyncProvider = new Provider('http://localhost:8011/')

    const PRIVATE_KEY: string = '0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c'

    const wallet = new Wallet(PRIVATE_KEY, zkSyncProvider)

    //   Deploy contract
    if (selectedContract == null) {
      remixClient.call(
        'notification' as any,
        'toast',
        'No contract selected'
      )

      return
    }

    const factory = new zksync.ContractFactory(
      selectedContract.abi,
      selectedContract.bytecode,
      wallet
    )

    try {
      const contract = await factory.deploy(...inputs)

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

      // @ts-expect-error
      contractOutputTx.customData.factoryDeps = '[ <...> ]'

      remixClient.terminal.log({
        value: `${JSON.stringify(contractOutputTx, null, 2)}`,
        type: 'info'
      })

      const deployedContract = {
        ...selectedContract,
        bytecode: selectedContract.bytecode,
        transactionHash: txHash,
        address
      } as DeployedContract

      deployedSetContracts([deployedContract, ...deployedContracts])
      deployedSetSelectedContract(deployedContract)

      setActiveTab('interaction')

      const transaction = {
        type: 'deploy',
        txId: txHash,
        env: 'local'
      } as Transaction

      setTransactions([transaction, ...transactions])
    } catch (e) {
      remixClient.emit('statusChanged', {
        key: 'failed',
        type: 'error',
        title: `Contract ${selectedContract.contractName} failed to deploy!`
      })

      remixClient.call(
        'notification' as any,
        'toast',
        `Error: ${(e as any).code}`
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
                          className="btn btn-primary btn-block d-block w-100 text-break mb-1 mt-2 px-0"
                          onClick={() => {
                            deploy()
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
