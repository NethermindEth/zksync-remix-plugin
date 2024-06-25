import React, { useEffect, useState } from 'react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { ethers } from 'ethers'
import {
  Environment,
  Compilation,
  Deployment,
  Interaction,
  TransactionHistory,
  SolidityVersion as CompilerVersion
} from '@/features'
import Accordian, { AccordianItem, AccordionContent, AccordionTrigger } from '@/ui_components/Accordian'
import StateAction from '@/components/StateAction'
import BackgroundNotices from '@/components/BackgroundNotices'
import { hashDirAtom, compilationAtom } from '@/atoms'
import { deploymentAtom } from '@/atoms/deployment'
import { initializeRemixClient, isLoadedAtom, remixClientAtom } from '@/stores/remixClient'
import storage from '@/utils/storage'
import useAsync from '@/hooks/useAsync'
import { type AccordianTabs } from '@/types/common'
import './styles.css'

export const Plugin = () => {
  const { status: compileStatus, errorMessages: compileErrorMessages } = useAtomValue(compilationAtom)
  const [isLoaded, setIsLoaded] = useAtom(isLoadedAtom)
  const setRemixClient = useSetAtom(remixClientAtom)
  const setHashDir = useSetAtom(hashDirAtom)

  useAsync(async () => {
    const client = await initializeRemixClient()
    setRemixClient(client)
    setIsLoaded(true)
  }, [setIsLoaded, setRemixClient])

  useEffect(() => {
    // read hashDir from localStorage
    const hashDir = storage.get('hashDir')
    if (hashDir != null) {
      setHashDir(hashDir)
    } else {
      // create a random hash of length 32
      const hashDir = ethers.utils.hashMessage(ethers.utils.randomBytes(32)).replace('0x', '')
      setHashDir(hashDir)
      storage.set('hashDir', hashDir)
    }
  }, [setHashDir])

  // Deployment Context state variables
  const { isDeploying, deployStatus } = useAtomValue(deploymentAtom)

  // Interaction state variables
  const [interactionStatus] = useState<'loading' | 'success' | 'error' | ''>('')

  const [currentAccordian, setCurrentAccordian] = useState<AccordianTabs>('compile')

  // eslint-disable-next-line @typescript-eslint/explicit-function-return-type
  const handleTabView = (clicked: AccordianTabs) => {
    if (currentAccordian === clicked) {
      setCurrentAccordian('')
    } else {
      setCurrentAccordian(clicked)
    }
  }
  return (
    //TODO: add a button for selecting the solidity version
    isLoaded ? (
      <>
        <div className="plugin-wrapper">
          <div className="plugin-main-wrapper">
            <CompilerVersion />
            <Accordian type="single" value={currentAccordian} defaultValue={'addRepository'}>
              <AccordianItem value="addRepository">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('addRepository')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                    <p style={{ all: 'unset' }}>Add Repository</p>
                    <StateAction
                      // TODO(edwin): add
                      value={''}
                    />
                  </span>
                </AccordionTrigger>
                <AccordionContent>
                  <RepositoryFetcher />
                </AccordionContent>
              </AccordianItem>
              <AccordianItem value="compile">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('compile')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                    <p style={{ all: 'unset' }}>Compile</p>
                    <StateAction
                      value={compileStatus === 'done' ? 'success' : compileStatus === 'failed' ? 'error' : ''}
                      errorTooltipText={
                        compileErrorMessages.length > 0
                          ? `${compileErrorMessages[0]} ${compileErrorMessages[1] ?? ''}. check terminal logs for more info.`
                          : ''
                      }
                    />
                  </span>
                </AccordionTrigger>
                <AccordionContent>
                  <Compilation setAccordian={setCurrentAccordian} />
                </AccordionContent>
              </AccordianItem>
              <AccordianItem value="deploy">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('deploy')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
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
              <AccordianItem value="interaction">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('interaction')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                    <p style={{ all: 'unset' }}>Interact</p>
                    <StateAction value={interactionStatus} />
                  </span>
                </AccordionTrigger>
                <AccordionContent>
                  <Interaction />
                </AccordionContent>
              </AccordianItem>

              {/*  Transactions start */}
              <AccordianItem value="transactions">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('transactions')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                    <p style={{ all: 'unset' }}> Transactions</p>
                  </span>
                </AccordionTrigger>
                <AccordionContent>
                  <TransactionHistory />
                </AccordionContent>
              </AccordianItem>
              <AccordianItem value="notices">
                <AccordionTrigger
                  onClick={() => {
                    handleTabView('notices')
                  }}
                >
                  <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                    <p style={{ all: 'unset' }}>Notices</p>
                  </span>
                </AccordionTrigger>
                <AccordionContent>
                  <BackgroundNotices />
                </AccordionContent>
              </AccordianItem>
            </Accordian>
          </div>
          <div>
            <Environment />
          </div>
        </div>
      </>
    ) : (
      <h1>Loading...</h1>
    )
  )
}
