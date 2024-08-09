import React, { useEffect, useState } from 'react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { ethers } from 'ethers'
import * as Tabs from '@radix-ui/react-tabs'
import { Compilation, Deployment, Interaction, TransactionHistory, Header, Footer, Environment } from '@/features'
import Accordian, { AccordianItem, AccordionContent, AccordionTrigger } from '@/ui_components/Accordian'
import StateAction from '@/components/StateAction'
import BackgroundNotices from '@/components/BackgroundNotices'
import { hashDirAtom, compilationAtom } from '@/atoms'
import { deploymentAtom } from '@/atoms/deployment'
import { initializeRemixClient, isLoadedAtom, remixClientAtom } from '@/stores/remixClient'
import storage from '@/utils/storage'
import useAsync from '@/hooks/useAsync'
import { type AccordianTabs } from '@/types/common'
import { FullScreenOverlay, Loader } from '@/ui_components'
import './styles.css'
import { Settings } from '@/components/Settings'
import { apiUrl } from '@/utils/network'

export const Plugin = () => {
  const { status: compileStatus, errorMessages: compileErrorMessages } = useAtomValue(compilationAtom)
  const [isLoaded, setIsLoaded] = useAtom(isLoadedAtom)
  const setRemixClient = useSetAtom(remixClientAtom)
  const setHashDir = useSetAtom(hashDirAtom)
  const { deployStatus } = useAtomValue(deploymentAtom)

  const [interactionStatus] = useState<'loading' | 'success' | 'error' | ''>('')

  const [currentAccordian, setCurrentAccordian] = useState<AccordianTabs>('compile')

  useAsync(async () => {
    const client = await initializeRemixClient()
    setRemixClient(client)
    setIsLoaded(true)

    try {
      const response = await fetch(`${apiUrl}/on-plugin-launched`, {
        method: 'POST',
        redirect: 'follow',
        headers: {
          'Content-Type': 'application/octet-stream'
        }
      })

      console.log('on-plugin-')

      if (!response.ok) {
        console.log('Could not post on launch')
      }
    } catch (error) {
      console.log('Could not post on launch', error)
    }
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
            <Header />
            <Environment />
            <Tabs.Root defaultValue="home" className="tabs-root">
              <Tabs.List
                className="flex justify-content-between rounded tab-list"
                aria-label="zksyc plugin tab options"
              >
                <div className="tabs-trigger" />
                <Tabs.Trigger value="home" className="tabs-trigger">
                  Home
                </Tabs.Trigger>
                <Tabs.Trigger value="transactions" className="tabs-trigger">
                  Transactions
                </Tabs.Trigger>
                <Tabs.Trigger value="info" className="tabs-trigger">
                  Info
                </Tabs.Trigger>
                <Tabs.Trigger value="settings" className="tabs-trigger">
                  Settings
                </Tabs.Trigger>
              </Tabs.List>
              <Tabs.Content value="home">
                <Accordian type="single" value={currentAccordian} defaultValue={'compile'}>
                  <AccordianItem value="compile">
                    <AccordionTrigger
                      onClick={() => {
                        handleTabView('compile')
                      }}
                    >
                      <span className="d-flex align-items-center" style={{ gap: '0.5rem' }}>
                        <span className={'accordian-list-number'}>1</span>
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
                        <span className={'accordian-list-number'}>2</span>
                        <p style={{ all: 'unset' }}>Deploy</p>
                        <StateAction
                          value={deployStatus === 'ERROR' ? 'error' : deployStatus === 'DONE' ? 'success' : ''}
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
                        <span className={'accordian-list-number'}>3</span>
                        <p style={{ all: 'unset' }}>Interact</p>
                        <StateAction value={interactionStatus} />
                      </span>
                    </AccordionTrigger>
                    <AccordionContent>
                      <Interaction />
                    </AccordionContent>
                  </AccordianItem>
                </Accordian>
              </Tabs.Content>
              <Tabs.Content value="transactions">
                <TransactionHistory />
              </Tabs.Content>
              <Tabs.Content value="info">
                <BackgroundNotices />
              </Tabs.Content>
              <Tabs.Content value="settings">
                <Settings />
              </Tabs.Content>
            </Tabs.Root>
          </div>
        </div>
        <Footer />
      </>
    ) : (
      <FullScreenOverlay>
        <Loader />
      </FullScreenOverlay>
    )
  )
}
