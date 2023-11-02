import React, { useContext, useEffect, useState } from 'react'

import { type BigNumberish, ethers } from 'ethers'
import CompiledContracts from '../../components/CompiledContracts'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import './styles.css'
import Container from '../../ui_components/Container'

import { ConnectionContext } from '../../contexts/ConnectionContext'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import { type AccordianTabs } from '../Plugin'
import DeploymentContext from '../../contexts/DeploymentContext'
import TransactionContext from '../../contexts/TransactionContext'
import { constants } from 'starknet'
import EnvironmentContext from '../../contexts/EnvironmentContext'
import { Wallet, Provider } from 'zksync-web3'
import * as zksync from 'zksync-web3'
import ConstructorInput from '../../components/ConstructorInput'
import { DeployedContractsContext } from '../../contexts/DeployedContractsContext'
import deployedContracts from '../../components/DeployedContracts'
import { type DeployedContract } from '../../types/contracts'

interface DeploymentProps {
  setActiveTab: (tab: AccordianTabs) => void
}

const Deployment: React.FC<DeploymentProps> = ({ setActiveTab }) => {
  const remixClient = useContext(RemixClientContext)
  const { account, provider } = useContext(ConnectionContext)
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
    remixClient.terminal.log('Hello!' as any)

    // TODO: Get provider
    const zkSyncProvider = new Provider('http://localhost:8011/')

    const PRIVATE_KEY: string = '0x3eb15da85647edd9a1159a4a13b9e7c56877c4eb33f614546d4db06a51868b1c'

    const wallet = new Wallet(PRIVATE_KEY, zkSyncProvider)

    //   Deploy contract

    const c = selectedContract

    if (c == null) {
      remixClient.terminal.log('No contract selected!' as any)

      return
    }

    const factory = new zksync.ContractFactory(
      c.abi,
      c.bytecode,
      wallet
    )

    remixClient.terminal.log('Started deploy!' as any)

    try {
      const contract = await factory.deploy(...inputs)

      remixClient.terminal.log('Deploying!' as any)
      remixClient.terminal.log('Args: ' + JSON.stringify(inputs) as any)

      const tx = await contract.deployed()

      console.log('Contract address:', tx.address)

      const address = tx.address
      const txHash = tx.deployTransaction.hash

      remixClient.terminal.log(`Contract address: ${tx.address}` as any)

      const deployedContract = {
        ...c,
        bytecode: c.bytecode,
        transactionHash: txHash,
        address
      } as DeployedContract

      deployedSetContracts([deployedContract, ...deployedContracts])
      deployedSetSelectedContract(deployedContract)

      setActiveTab('interaction')
    } catch (e) {
      remixClient.terminal.log(`Error: ${e}` as any)
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
