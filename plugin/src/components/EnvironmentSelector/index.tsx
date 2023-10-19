import React, { useContext } from 'react'
import { devnets } from '../../utils/network'
import { ConnectionContext } from '../../contexts/ConnectionContext'
import './styles.css'
import EnvironmentContext from '../../contexts/EnvironmentContext'


const EnvironmentSelector = () => {
  const { setProvider } = useContext(ConnectionContext)
  const { env, setEnv, setDevnet } = useContext(EnvironmentContext)




  async function handleEnvironmentChange (event: any): Promise<void> {
    const value = parseInt(event.target.value)
    if (value > 0) {
      setDevnet(devnets[value - 1])
      if (value === 2) setEnv('remoteDevnet')
      else setEnv('localDevnet')
      setProvider(null)
      return
    }
    setEnv('wallet')
  }

  const getDefualtIndex = (): number => {
    if (env === 'wallet') return 0
    if (env === 'localDevnet') return 1
    return 2
  }

  return (
    <div className="environment-selector-wrapper">
      <select
        className="custom-select"
        aria-label=".form-select-sm example"
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        onChange={handleEnvironmentChange}
        defaultValue={getDefualtIndex()}
      >
        {devnets.reduce<JSX.Element[]>(
          (acc, devnet, index) => {
            acc.push(
              <option value={index + 1} key={index + 1}>
                {devnet.name}
              </option>
            )
            return acc
          },
          [
            <option value={0} key={0}>
              Wallet Selection
            </option>
          ]
        )}
      </select>
    </div>
  )
}

export default EnvironmentSelector
