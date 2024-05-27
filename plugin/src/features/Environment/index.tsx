/* eslint-disable multiline-ternary */
import React, { useState } from 'react'
import DevnetAccountSelector from '../../components/DevnetAccountSelector'
import './styles.css'
import EnvironmentSelector from '../../components/EnvironmentSelector'
import Wallet from '../../components/Wallet'
import ManualAccountComp from '../../components/ManualAccount'
import { RxDotFilled } from 'react-icons/rx'
import Accordian, {
  AccordianItem,
  AccordionContent,
  AccordionTrigger
} from '../../ui_components/Accordian'
import { useAtom, useAtomValue } from 'jotai'
import { envAtom, isDevnetAliveAtom } from '../../atoms/environment'
import { type EnvType } from '../../types/transaction'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface EnvironmentProps {}

export const Environment: React.FC<EnvironmentProps> = () => {
  const [env, setEnv] = useAtom(envAtom)
  const isDevnetAlive = useAtomValue(isDevnetAliveAtom)
  const [prevEnv, setPrevEnv] = useState<EnvType>(env)

  const [currentPane, setCurrentPane] = useState('environment')

  return (
    <div className="zksync-connection-component mb-8">
      <Accordian type="single" value={currentPane} defaultValue={'environment'}>
        <AccordianItem value="environment">
          <AccordionTrigger
            onClick={() => {
              setCurrentPane(currentPane === 'environment' ? '' : 'environment')
            }}
          >
            <div className="trigger-env">
              <p>Environment</p>
              <button
                type="button"
                className="mb-0 btn float-right rounded-pill env-testnet-btn"
                onClick={(e) => {
                  e.stopPropagation()
                  if (env !== 'manual') setPrevEnv(env)
                  setEnv('manual')
                }}
              >
                Test Accounts
              </button>
            </div>
          </AccordionTrigger>
          <AccordionContent>
            <>
              <div className="flex flex-column">
                {env !== 'manual' ? (
                  <>
                    <div className="flex flex-column">
                      <label className="">Environment selection</label>
                      <div className="flex_dot">
                        <EnvironmentSelector />
                        {env === 'wallet' ? (
                          <RxDotFilled
                            size={'30px'}
                            color="rebeccapurple"
                            title="Wallet is active"
                          />
                        ) : isDevnetAlive ? (
                          <RxDotFilled
                            size={'30px'}
                            color="lime"
                            title="Devnet is live"
                          />
                        ) : (
                          <RxDotFilled
                            size={'30px'}
                            color="red"
                            title="Devnet server down"
                          />
                        )}
                      </div>
                    </div>
                    <div className="flex flex-column">
                      {['localDevnet', 'remoteDevnet'].includes(env) ? (
                        <DevnetAccountSelector />
                      ) : (
                        <Wallet />
                      )}
                    </div>
                  </>
                ) : (
                  <ManualAccountComp prevEnv={prevEnv} />
                )}
              </div>
            </>
          </AccordionContent>
        </AccordianItem>
      </Accordian>
    </div>
  )
}
