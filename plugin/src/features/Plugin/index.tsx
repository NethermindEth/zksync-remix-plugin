import React, { useEffect, useState } from 'react'
import { Environment } from '../Environment'
import './styles.css'
import Compilation from '../Compilation'
import Deployment from '../Deployment'
import Interaction from '../Interaction'
import Accordian, { AccordianItem, AccordionContent, AccordionTrigger } from '../../ui_components/Accordian'
import TransactionHistory from '../TransactionHistory'
import CompilerVersion from '../CompilerVersion'
import StateAction from '../../components/StateAction'
import BackgroundNotices from '../../components/BackgroundNotices'
import { useAtomValue, useSetAtom } from 'jotai'
import { isCompilingAtom, statusAtom as compilationStatusAtom } from '../../atoms/compilation'
import { deploymentAtom } from '../../atoms/deployment'
import useRemixClient from '../../hooks/useRemixClient'
import { pluginLoaded } from '../../atoms/remixClient'

export type AccordianTabs =
  | 'compile'
  | 'deploy'
  | 'interaction'
  | 'transactions'
  | ''

const Plugin: React.FC = () => {
  // Compilation Context state variables
  const compilationStatus = useAtomValue(compilationStatusAtom)
  const isCompiling = useAtomValue(isCompilingAtom)

  // Deployment Context state variables
  const {
    isDeploying,
    deployStatus
  } = useAtomValue(deploymentAtom)

  // Interaction state variables
  const [interactionStatus, setInteractionStatus] = useState<'loading' | 'success' | 'error' | ''>('')

  const [currentAccordian, setCurrentAccordian] =
    useState<AccordianTabs>('compile')

  const setPluginLoaded = useSetAtom(pluginLoaded)
  const { remixClient } = useRemixClient()

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    const id = setTimeout(async (): Promise<void> => {
      await remixClient.onload(() => {
        setPluginLoaded(true)
      })
    }, 1)
    return () => {
      clearInterval(id)
    }
  }, [])

  // eslint-disable-next-line @typescript-eslint/explicit-function-return-type
  const handleTabView = (clicked: AccordianTabs) => {
    if (currentAccordian === clicked) {
      setCurrentAccordian('')
    } else {
      setCurrentAccordian(clicked)
    }
  }

  return (
    // add a button for selecting the solidity version
    <>
      <div className='plugin-wrapper'>
        <div className='plugin-main-wrapper'>
          <CompilerVersion />
          <Accordian
            type='single'
            value={currentAccordian}
            defaultValue={'compile'}

          >
            <AccordianItem value='compile'>
              <AccordionTrigger
                onClick={() => {
                  handleTabView('compile')
                }}
              >
                                <span
                                  className='d-flex align-items-center'
                                  style={{ gap: '0.5rem' }}
                                >
                                  <p style={{ all: 'unset' }}>Compile</p>
                                  <StateAction
                                    value={
                                      isCompiling
                                        ? 'loading'
                                        : compilationStatus === 'done'
                                          ? 'success'
                                          : compilationStatus === 'failed' ? 'error' : ''
                                    }
                                  />
                                </span>
              </AccordionTrigger>
              <AccordionContent>
                <Compilation setAccordian={setCurrentAccordian} />
              </AccordionContent>
            </AccordianItem>
            <AccordianItem value='deploy'>
              <AccordionTrigger
                onClick={() => {
                  handleTabView('deploy')
                }}
              >
                                <span
                                  className='d-flex align-items-center'
                                  style={{ gap: '0.5rem' }}
                                >
                                  <p style={{ all: 'unset' }}>Deploy</p>
                                  <StateAction
                                    value={
                                      isDeploying
                                        ? 'loading'
                                        : deployStatus === 'error'
                                          ? 'error'
                                          : deployStatus === 'done'
                                            ? 'success'
                                            : ''
                                    }
                                  />
                                </span>
              </AccordionTrigger>
              <AccordionContent>
                <Deployment setActiveTab={setCurrentAccordian} />
              </AccordionContent>
            </AccordianItem>
            <AccordianItem value='interaction'>
              <AccordionTrigger
                onClick={() => {
                  handleTabView('interaction')
                }}
              >
                                <span
                                  className='d-flex align-items-center'
                                  style={{ gap: '0.5rem' }}
                                >
                                  <p style={{ all: 'unset' }}>Interact</p>
                                  <StateAction
                                    value={interactionStatus}
                                  />
                                </span>
              </AccordionTrigger>
              <AccordionContent>
                <Interaction setInteractionStatus={setInteractionStatus} />
              </AccordionContent>
            </AccordianItem>

            {/*  Transactions start */}
            <AccordianItem value='transactions'>
              <AccordionTrigger
                onClick={() => {
                  handleTabView('transactions')
                }}
              >
                              <span
                                className='d-flex align-items-center'
                                style={{ gap: '0.5rem' }}
                              >
                                <p style={{ all: 'unset' }}> Transactions</p>
                              </span>
              </AccordionTrigger>
              <AccordionContent>
                <TransactionHistory />
              </AccordionContent>
            </AccordianItem>
          </Accordian>
          <div className='mt-5'>
            <BackgroundNotices />
          </div>
        </div>
        <div>
          <Environment />
        </div>

      </div>
    </>
  )
}

export default Plugin
