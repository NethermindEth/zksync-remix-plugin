/* eslint-disable multiline-ternary */
import React, { useContext, useState } from 'react'
import DevnetAccountSelector from '../../components/DevnetAccountSelector'
import './styles.css'
import EnvironmentSelector from '../../components/EnvironmentSelector'
import Wallet from '../../components/Wallet'
import { RxDotFilled } from 'react-icons/rx'
import Accordian, {
  AccordianItem,
  AccordionContent,
  AccordionTrigger
} from '../../ui_components/Accordian'
import ManualAccount from '../../components/ManualAccount'
import useRemixClient from '../../hooks/useRemixClient'
import { useAtom } from 'jotai/react/useAtom'
import { accountAtom, providerAtom } from '../../atoms/connection'
import { useSetAtom } from 'jotai/react/useSetAtom'
import { envAtom, isDevnetAliveAtom } from '../../atoms/environment'
import { useAtomValue } from 'jotai/react/useAtomValue'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface EnvironmentProps {}

const Environment: React.FC<EnvironmentProps> = () => {
  // Using the context
  const { remixClient } = useRemixClient()
  const setAccount = useSetAtom(accountAtom)
  const setProvider = useSetAtom(providerAtom)

  const [env, setEnv] = useAtom(envAtom)
  const isDevnetAlive = useAtomValue(isDevnetAliveAtom)
  const [prevEnv, setPrevEnv] = useState<string>(env)

  const [currentPane, setCurrentPane] = useState('environment')

  return (
    <div className="zksync-connection-component mb-8">
      <Accordian type="single" value={currentPane} defaultValue={'environment'}>
        <AccordianItem value="environment">
          <AccordionTrigger
            onClick={() => { setCurrentPane(currentPane === 'environment' ? '' : 'environment') }
            }
          >
            <div className="trigger-env">
              <p>Environment</p>
              {/* Select test accounts */}
              <button
                type="button"
                className="mb-0 btn btn-sm btn-outline-secondary float-right rounded-pill env-testnet-btn"
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
              <div className="flex">
                {env !== 'manual' ? (
                  <>
                    <div className="flex">
                      <label className="">Environment selection</label>
                      <div className="flex_dot">
                        <EnvironmentSelector />
                        {env === 'wallet'
                          ? (
                          <RxDotFilled
                            size={'30px'}
                            color="rebeccapurple"
                            title="Wallet is active"
                          />
                            )
                          : isDevnetAlive
                            ? (
                          <RxDotFilled
                            size={'30px'}
                            color="lime"
                            title="Devnet is live"
                          />
                              )
                            : (
                          <RxDotFilled
                            size={'30px'}
                            color="red"
                            title="Devnet server down"
                          />
                              )}
                      </div>
                    </div>
                    <div className="flex">
                      {['localDevnet', 'remoteDevnet'].includes(env) ? (
                        <DevnetAccountSelector />
                      ) : (
                        <Wallet />
                      )}
                    </div>
                  </>
                ) : (
                  <ManualAccount prevEnv={prevEnv}/>
                )}
              </div>
            </>
          </AccordionContent>
        </AccordianItem>
      </Accordian>
    </div>
  )
}

export { Environment }
