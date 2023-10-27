import React, { useContext, useEffect, useState } from 'react'

import { type BigNumberish } from 'ethers'
import CompiledContracts from '../../components/CompiledContracts'
import { CompiledContractsContext } from '../../contexts/CompiledContractsContext'
import {
  type CallDataObj,
  type CallDataObject,
  type Contract
} from '../../types/contracts'
import { getConstructor, getParameterType } from '../../utils/utils'
import './styles.css'
import Container from '../../ui_components/Container'

import { ConnectionContext } from '../../contexts/ConnectionContext'
import { RemixClientContext } from '../../contexts/RemixClientContext'
import { type AccordianTabs } from '../Plugin'
import DeploymentContext from '../../contexts/DeploymentContext'
import TransactionContext from '../../contexts/TransactionContext'
import { constants } from 'starknet'
import EnvironmentContext from '../../contexts/EnvironmentContext'

interface DeploymentProps {
  setActiveTab: (tab: AccordianTabs) => void
}

const Deployment: React.FC<DeploymentProps> = ({ setActiveTab }) => {
  const remixClient = useContext(RemixClientContext)
  const { account, provider } = useContext(ConnectionContext)
  const { contracts, selectedContract, setContracts, setSelectedContract } =
    useContext(CompiledContractsContext)

  const [constructorCalldata, setConstructorCalldata] =
    useState<CallDataObject>({})

  const {
    isDeploying,
    setIsDeploying,
    deployStatus,
    setDeployStatus,
    constructorInputs,
    setConstructorInputs,
    notEnoughInputs,
    setNotEnoughInputs
  } = useContext(DeploymentContext)

  const { transactions, setTransactions } = useContext(TransactionContext)
  const { env } = useContext(EnvironmentContext)

  const [chainId, setChainId] = useState<constants.StarknetChainId>(
    constants.StarknetChainId.SN_GOERLI
  )

  return (
    <>
      <Container>
        {contracts.length > 0 && selectedContract != null
          ? (
          // <div className="">
          //   <CompiledContracts show={'class'} />
          //   <form onSubmit={handleDeploySubmit}>
          //     {constructorInputs.map((input, index) => {
          //       return (
          //         <div
          //           className="udapp_multiArg constructor-label-wrapper"
          //           key={index}
          //         >
          //           <label key={index} className="constructor-label">
          //             {`${input.name} (${
          //               getParameterType(input.type) ?? ''
          //             }): `}
          //           </label>
          //           <input
          //             className="form-control constructor-input"
          //             name={input.name}
          //             data-type={input.type}
          //             data-index={index}
          //             value={constructorCalldata[index]?.value ?? ''}
          //             onChange={handleConstructorCalldataChange}
          //           />
          //         </div>
          //       )
          //     })}
          //     <button
          //       className="btn btn-primary btn-block d-block w-100 text-break remixui_disabled mb-1 mt-3 px-0"
          //       style={{
          //         cursor: `${
          //           isDeploying ||
          //           account == null ||
          //           selectedContract.deployedInfo.some(
          //             (info) =>
          //               info.address === account.address &&
          //               info.chainId === chainId
          //           )
          //             ? 'not-allowed'
          //             : 'pointer'
          //         }`
          //       }}
          //       disabled={
          //         isDeploying ||
          //         account == null ||
          //         selectedContract.deployedInfo.some(
          //           (info) =>
          //             info.address === account.address &&
          //             info.chainId === chainId
          //         )
          //       }
          //       aria-disabled={
          //         isDeploying ||
          //         account == null ||
          //         selectedContract.deployedInfo.some(
          //           (info) =>
          //             info.address === account.address &&
          //             info.chainId === chainId
          //         )
          //       }
          //       type="submit"
          //     >
          //       <div className="d-flex align-items-center justify-content-center">
          //         <div className="text-truncate overflow-hidden text-nowrap">
          //           {isDeploying
          //             ? (
          //             <>
          //               <span
          //                 className="spinner-border spinner-border-sm"
          //                 role="status"
          //                 aria-hidden="true"
          //               >
          //                 {' '}
          //               </span>
          //               <span style={{ paddingLeft: '0.5rem' }}>
          //                 {deployStatus}
          //               </span>
          //             </>
          //               )
          //             : (
          //             <div className="text-truncate overflow-hidden text-nowrap">
          //               {account != null &&
          //               selectedContract.deployedInfo.some(
          //                 (info) =>
          //                   info.address === account.address &&
          //                   info.chainId === chainId
          //               )
          //                 ? (
          //                 <span>
          //                   {' '}
          //                   Deployed <i className="bi bi-check"></i>{' '}
          //                   {selectedContract.name}
          //                 </span>
          //                   )
          //                 : (
          //                 <span> Deploy {selectedContract.name}</span>
          //                   )}
          //             </div>
          //               )}
          //         </div>
          //       </div>
          //     </button>
          //   </form>
          //   {account != null &&
          //     selectedContract.deployedInfo.some(
          //       (info) =>
          //         info.address === account.address && info.chainId === chainId
          //     ) && (
          //       <div className="mt-3">
          //         <label style={{ display: 'block' }}>
          //           Contract deployed! See{' '}
          //           <a
          //             href="/"
          //             className="text-info"
          //             onClick={(e) => {
          //               e.preventDefault()
          //               setActiveTab('interaction')
          //             }}
          //           >
          //             Interact
          //           </a>{' '}
          //           for more!
          //         </label>
          //       </div>
          //   )}
          //   {notEnoughInputs && (
          //     <label>Please fill out all constructor fields!</label>
          //   )}
          // </div>
                <div></div>
            )
          : (
          <p>No contracts ready for deployment yet, compile a solidity contract</p>
            )}
      </Container>
    </>
  )
}

export default Deployment
