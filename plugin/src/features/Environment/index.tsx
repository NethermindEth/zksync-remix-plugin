import React from 'react'
import { useAtom } from 'jotai'
import * as Tabs from '@radix-ui/react-tabs'
import {
  AccountSelector,
  EnvironmentSelector,
  Wallet,
  ManualAccount,
  DevnetStatus,
  CurrentEnv,
  CustomNetwork
} from '@/components'
import Accordian, { AccordianItem, AccordionContent, AccordionTrigger } from '@/ui_components/Accordian'
import { envAtom } from '@/atoms/environment'
import './styles.css'

export const Environment = () => {
  const [env, setEnv] = useAtom(envAtom)

  return (
    <Accordian className={'accordian-env'} type={'single'} defaultValue={'closed'}>
      <AccordianItem value={'closed'}></AccordianItem>
      <AccordianItem value={'env'} className={'accordian-item-env'}>
        <AccordionTrigger className={'accordian-trigger-env'}>
          <CurrentEnv />
        </AccordionTrigger>

        <AccordionContent className={'accordian-content-env'}>
          <div className="zksync-connection-component">
            <Tabs.Root
              defaultValue={env === 'manual' ? 'manual-accounts' : 'environment'}
              onValueChange={(e: any) => {
                if (e === 'environment') {
                  setEnv('remoteDevnet')
                } else {
                  setEnv('manual')
                }
              }}
            >
              <Tabs.List className={'flex justify-content-center rounded tab-list tab-header-env'}>
                <Tabs.Trigger className={'tabs-trigger'} value={'environment'}>
                  Environment
                </Tabs.Trigger>
                <Tabs.Trigger className={'tabs-trigger'} value={'manual-accounts'}>
                  Manual Accounts
                </Tabs.Trigger>
              </Tabs.List>

              <Tabs.Content value={'environment'} className={'tabs-content-env'}>
                <div>
                  <div className="flex flex-column">
                    {env !== 'manual' ? (
                      <div>
                        <div className="flex flex-column">
                          <label className="">Environment selection</label>
                          <div className="flex_dot">
                            <div className={'env-selector-wrapper'}>
                              <EnvironmentSelector />
                            </div>
                            <DevnetStatus />
                          </div>
                        </div>
                        <div className="flex flex-column">
                          {['localDevnet', 'remoteDevnet'].includes(env) && <AccountSelector accountsType="devnet" />}
                          {env === 'wallet' && <Wallet />}
                          {env === 'customNetwork' && <CustomNetwork />}
                        </div>
                      </div>
                    ) : (
                      <div />
                    )}
                  </div>
                </div>
              </Tabs.Content>
              <Tabs.Content value={'manual-accounts'} className="tabs-content-env">
                <ManualAccount />
              </Tabs.Content>
            </Tabs.Root>
          </div>
        </AccordionContent>
      </AccordianItem>
    </Accordian>
  )
}
