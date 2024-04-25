/* eslint-disable @typescript-eslint/restrict-plus-operands */
/* eslint-disable @typescript-eslint/strict-boolean-expressions */
import React, { useEffect } from 'react'
import { apiUrl } from '../../utils/network'
import { getFileExtension, getFileNameFromPath } from '../../utils/utils'
import './styles.css'
import Container from '../../ui_components/Container'
import storage from '../../utils/storage'
import { ethers } from 'ethers'
import { type AccordianTabs } from '../Plugin'
import { type VerificationResult } from '../../types/contracts'
import { asyncPost } from '../../utils/async_api_requests'
import {
  activeTomlPathAtom,
  verificationAtom,
  currentFilenameAtom,
  hashDirAtom,
  isVerifyingAtom,
  isValidSolidityAtom,
  noFileSelectedAtom,
  statusAtom,
  tomlPathsAtom
} from '../../atoms/verification'
import { useAtomValue, useSetAtom } from 'jotai'
import { solidityVersionAtom } from '../../atoms/version'
import useRemixClient from '../../hooks/useRemixClient'
import { providerAtom } from '../../atoms/connection'
import { envAtom } from '../../atoms/environment'
import ConstructorInput from '../../components/ConstructorInput'
import { selectedContractAtom } from '../../atoms/compiledContracts'

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface VerificationProps {
  setAccordian: React.Dispatch<React.SetStateAction<AccordianTabs>>
}

const Verification: React.FC<VerificationProps> = ({ setAccordian }) => {
  const { remixClient } = useRemixClient()

  const {
    status,
    currentFilename,
    isVerifying,
    isValidSolidity,
    noFileSelected,
    hashDir,
    tomlPaths,
    activeTomlPath
  } = useAtomValue(verificationAtom)

  const setStatus = useSetAtom(statusAtom)
  const setCurrentFilename = useSetAtom(currentFilenameAtom)
  const setIsVerifying = useSetAtom(isVerifyingAtom)
  const setNoFileSelected = useSetAtom(noFileSelectedAtom)
  const setHashDir = useSetAtom(hashDirAtom)
  const setTomlPaths = useSetAtom(tomlPathsAtom)
  const setActiveTomlPath = useSetAtom(activeTomlPathAtom)
  const setIsValidSolidity = useSetAtom(isValidSolidityAtom)

  const solidityVersion = useAtomValue(solidityVersionAtom)
  const provider = useAtomValue(providerAtom)
  const env = useAtomValue(envAtom)
  const selectedContract = useAtomValue(selectedContractAtom)

  const [currWorkspacePath, setCurrWorkspacePath] = React.useState<string>('')
  const [contractAddress, setContractAddress] = React.useState<string>('')
  const [selectedChainName, setSelectedChainName] = React.useState<string | undefined>()
  const [inputs, setInputs] = React.useState<string[]>([])

  useEffect(() => {
    const constructor = selectedContract?.abi.find((abiElement) => {
      return abiElement.type === 'constructor'
    })

    if (constructor === undefined || constructor?.inputs === undefined) {
      setInputs([])
      return
    }

    setInputs(new Array(constructor?.inputs.length).fill(''))
  }, [selectedContract])

  useEffect(() => {
    // read hashDir from localStorage
    const hashDir = storage.get('hashDir')
    if (hashDir != null) {
      setHashDir(hashDir)
    } else {
      // create a random hash of length 32
      const hashDir = ethers.utils
        .hashMessage(ethers.utils.randomBytes(32))
        .replace('0x', '')
      setHashDir(hashDir)
      storage.set('hashDir', hashDir)
    }
  }, [hashDir])

  useEffect(() => {
    let name: string | undefined
    if (provider?.network?.chainId === 300) name = 'sepolia'
    if (provider?.network?.chainId === 324) name = 'mainnet'
    setSelectedChainName(name)
    console.log(`effect ran! ${name ?? 'undefined'}`)
  }, [provider, env])

  useEffect(() => {
    remixClient.on('fileManager', 'noFileSelected', () => {
      setNoFileSelected(true)
    })
  }, [remixClient])

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      try {
        if (noFileSelected) {
          throw new Error('No file selected')
        }

        // get current file
        const currentFile = await remixClient.call(
          'fileManager',
          'getCurrentFile'
        )
        if (currentFile.length > 0) {
          const filename = getFileNameFromPath(currentFile)
          const currentFileExtension = getFileExtension(filename)
          setIsValidSolidity(currentFileExtension === 'sol')
          setCurrentFilename(filename)

          remixClient.emit('statusChanged', {
            key: 'succeed',
            type: 'info',
            title: 'Current file: ' + currentFilename
          })
        }
      } catch (e) {
        remixClient.emit('statusChanged', {
          key: 'failed',
          type: 'info',
          title: 'Please open a solidity file to verify'
        })
        console.log('error: ', e)
      }
    }, 500)
  }, [remixClient, currentFilename, noFileSelected])

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      remixClient.on(
        'fileManager',
        'currentFileChanged',
        (currentFileChanged: any) => {
          const filename = getFileNameFromPath(currentFileChanged)
          const currentFileExtension = getFileExtension(filename)
          setIsValidSolidity(currentFileExtension === 'sol')
          setCurrentFilename(filename)
          remixClient.emit('statusChanged', {
            key: 'succeed',
            type: 'info',
            title: 'Current file: ' + currentFilename
          })
          setNoFileSelected(false)
        }
      )
    }, 500)
  }, [remixClient, currentFilename])

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      try {
        if (noFileSelected) {
          throw new Error('No file selected')
        }
        const currentFilePath = await remixClient.call(
          'fileManager',
          'getCurrentFile'
        )
        if (!currentFilePath.endsWith('.sol')) {
          throw new Error('Not a valid solidity file')
        }
        const currentFileContent = await remixClient.call(
          'fileManager',
          'readFile',
          currentFilePath
        )
        await fetch(`${apiUrl}/save_code/${hashDir}/${currentFilePath}`, {
          method: 'POST',
          body: currentFileContent,
          redirect: 'follow',
          headers: {
            'Content-Type': 'application/octet-stream'
          }
        })
      } catch (e) {
        remixClient.emit('statusChanged', {
          key: 'failed',
          type: 'info',
          title: 'Please open a solidity file to verify'
        })
        console.log('error: ', e)
      }
    }, 100)
  }, [currentFilename, remixClient])

  async function getTomlPaths (
    workspacePath: string,
    currPath: string
  ): Promise<string[]> {
    const resTomlPaths: string[] = []

    try {
      const allFiles = await remixClient.fileManager.readdir(
        workspacePath + '/' + currPath
      )
      // get keys of allFiles object
      const allFilesKeys = Object.keys(allFiles)
      // const get all values of allFiles object
      const allFilesValues = Object.values(allFiles)

      for (let i = 0; i < allFilesKeys.length; i++) {
        if (allFilesKeys[i].endsWith('Scarb.toml')) {
          resTomlPaths.push(currPath)
        }
        if (Object.values(allFilesValues[i])[0]) {
          const recTomlPaths = await getTomlPaths(
            workspacePath,
            allFilesKeys[i]
          )
          resTomlPaths.push(...recTomlPaths)
        }
      }
    } catch (e) {
      console.log('error: ', e)
    }
    return resTomlPaths
  }

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      // get current workspace path
      try {
        const currWorkspace = await remixClient.filePanel.getCurrentWorkspace()
        setCurrWorkspacePath(currWorkspace.absolutePath)
      } catch (e) {
        console.log('error: ', e)
      }
    })
  })

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      // get current workspace path
      try {
        if (currWorkspacePath === '') return
        const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

        setTomlPaths(allTomlPaths)
        if (activeTomlPath === '' || activeTomlPath === undefined) {
          setActiveTomlPath(tomlPaths[0])
        }
      } catch (e) {
        console.log('error: ', e)
      }
    }, 100)
  }, [currWorkspacePath])

  useEffect(() => {
    if (activeTomlPath === '' || activeTomlPath === undefined) {
      setActiveTomlPath(tomlPaths[0])
    }
  }, [tomlPaths])

  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    setTimeout(async () => {
      remixClient.on('fileManager', 'fileAdded', (_: any) => {
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        setTimeout(async () => {
          // get current workspace path
          try {
            const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

            setTomlPaths(allTomlPaths)
          } catch (e) {
            console.log('error: ', e)
          }
        }, 100)
      })
      remixClient.on('fileManager', 'folderAdded', (_: any) => {
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        setTimeout(async () => {
          // get current workspace path
          try {
            const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

            setTomlPaths(allTomlPaths)
          } catch (e) {
            console.log('error: ', e)
          }
        }, 100)
      })
      remixClient.on('fileManager', 'fileRemoved', (_: any) => {
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        setTimeout(async () => {
          // get current workspace path
          try {
            const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

            setTomlPaths(allTomlPaths)
          } catch (e) {
            console.log('error: ', e)
          }
        }, 100)
      })
      remixClient.on('filePanel', 'workspaceCreated', (_: any) => {
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        setTimeout(async () => {
          // get current workspace path
          try {
            const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

            setTomlPaths(allTomlPaths)
          } catch (e) {
            console.log('error: ', e)
          }
        }, 100)
      })
      remixClient.on('filePanel', 'workspaceRenamed', (_: any) => {
        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        setTimeout(async () => {
          // get current workspace path
          try {
            const allTomlPaths = await getTomlPaths(currWorkspacePath, '')

            setTomlPaths(allTomlPaths)
          } catch (e) {
            console.log('error: ', e)
          }
        }, 100)
      })
    }, 500)
  }, [remixClient])

  const verifications = [
    {
      validation: isValidSolidity,
      isLoading: isVerifying,
      onClick: verify
    }
  ]

  async function verify (): Promise<void> {
    setIsVerifying(true)
    setStatus('Verifying...')
    // clear current file annotations: inline syntax error reporting
    await remixClient.editor.clearAnnotations()
    try {
      setStatus('Getting solidity file path...')
      const currentFilePath = await remixClient.call(
        'fileManager',
        'getCurrentFile'
      )

      setStatus('Getting solidity file content...')
      const currentFileContent = await remixClient.call(
        'fileManager',
        'readFile',
        currentFilePath
      )

      setStatus('Parsing solidity code...')
      let response = await fetch(
        `${apiUrl}/save_code/${solidityVersion}/${hashDir}/${currentFilePath}`,
        {
          method: 'POST',
          body: currentFileContent,
          redirect: 'follow',
          headers: {
            'Content-Type': 'application/octet-stream'
          }
        }
      )

      if (!response.ok) {
        await remixClient.call(
          'notification' as any,
          'toast',
          'Could not reach solidity verification server'
        )
        throw new Error('Solidity Verification Request Failed')
      }

      setStatus('Verifying...')

      const chainName = selectedChainName ?? 'unknown'

      response = await asyncPost(
        `verify-async/${solidityVersion}/${chainName}/${contractAddress}/${hashDir}/${currentFilePath}`,
        'verify-result',
        inputs
      )

      if (!response.ok) {
        await remixClient.call(
          'notification' as any,
          'toast',
          'Could not reach solidity verification server'
        )
        throw new Error('Solidity Verification Request Failed')
      }

      // get Json body from response
      const verificationResult = JSON.parse(await response.text()) as VerificationResult

      if (verificationResult.status !== 'Success') {
        setStatus('Reporting Errors...')
        await remixClient.terminal.log({
          value: verificationResult.message,
          type: 'error'
        })

        const errorLets = verificationResult.message.trim().split('\n')

        // remove last element if it's starts with `Error:`
        if (errorLets[errorLets.length - 1].startsWith('Error:')) {
          errorLets.pop()
        }

        // break the errorLets in array of arrays with first element contains the string `Plugin diagnostic`
        const errorLetsArray = errorLets.reduce(
          (acc: any, curr: any) => {
            if (curr.startsWith('error:') || curr.startsWith('warning:')) {
              acc.push([curr])
            } else {
              acc[acc.length - 1].push(curr)
            }
            return acc
          },
          [['errors diagnostic:']]
        )

        // remove the first array
        errorLetsArray.shift()

        // eslint-disable-next-line @typescript-eslint/no-misused-promises
        errorLetsArray.forEach(async (errorLet: any) => {
          const errorType = errorLet[0].split(':')[0].trim()
          const errorTitle = errorLet[0].split(':').slice(1).join(':').trim()
          const errorLine = errorLet[1].split(':')[1].trim()
          const errorColumn = errorLet[1].split(':')[2].trim()
          // join the rest of the array
          const errorMsg = errorLet.slice(2).join('\n')

          await remixClient.editor.addAnnotation({
            row: Number(errorLine) - 1,
            column: Number(errorColumn) - 1,
            text: errorMsg + '\n' + errorTitle,
            type: errorType
          })
        })

        // trim sierra message to get last line
        const lastLine = verificationResult.message.trim().split('\n').pop()?.trim()

        remixClient.emit('statusChanged', {
          key: 'failed',
          type: 'error',
          title: (lastLine ?? '').startsWith('Error') ? lastLine : 'Verification Failed'
        })
        throw new Error(
          'Solidity Verification Failed, logs can be read in the terminal log'
        )
      } else {
        remixClient.emit('statusChanged', {
          key: 'succeed',
          type: 'success',
          title: 'Verification Successful'
        })

        await remixClient.terminal.log({
          value: 'Verification successful.',
          type: 'info'
        })

        await remixClient.call(
          'notification' as any,
          'toast',
          'Verification successful.'
        )
      }
    } catch (e) {
      setStatus('failed')
      if (e instanceof Error) {
        await remixClient.call('notification' as any, 'alert', {
          id: 'zksyncRemixPluginAlert',
          title: 'Expectation Failed',
          message: e.message
        })
      }
      console.error(e)
    }
    setIsVerifying(false)
  }

  const verificationCard = (
    validation: boolean,
    isLoading: boolean,
    onClick: () => unknown
  ): React.ReactElement => {
    return (
      <Container>
        {activeTomlPath !== '' && activeTomlPath !== undefined && tomlPaths?.length > 0 && (
          <div className='project-dropdown-wrapper d-flex flex-column mb-3'>
            <div className='mx-auto'>Verify a single Solidity contract:</div>
          </div>
        )}
        <div className='input-field'>
          <label htmlFor='input-contract-address' className='input-label'>Contract Address</label>
          <input
            type='text'
            id='input-contract-address'
            value={contractAddress}
            onChange={(e) => { setContractAddress(e.target.value) }}
            className='input-text'
            placeholder={'Contract Address'}
          />
        </div>
        <ConstructorInput inputs={inputs} setInputs={setInputs}></ConstructorInput>
        <button
          className='btn btn-primary w-100 text-break remixui_disabled mb-1 mt-1 px-0'
          style={{
            cursor: `${
              !validation || !currentFilename || isVerifying || !selectedChainName ? 'not-allowed' : 'pointer'
            }`
          }}
          disabled={!validation || !currentFilename || isVerifying || !selectedChainName}
          aria-disabled={!validation || !currentFilename || isVerifying || !selectedChainName}
          onClick={onClick}
        >
          <div className='d-flex align-items-center justify-content-center'>
            <div className='text-truncate overflow-hidden text-nowrap'>
              {!validation
                ? (
                  <span>Select a valid solidity file</span>
                  )
                : (
                  <>
                    <div className='d-flex align-items-center justify-content-center'>
                      {isLoading
                        ? (
                          <>
                        <span
                          className='spinner-border spinner-border-sm'
                          role='status'
                          aria-hidden='true'
                        >
                          {' '}
                        </span>
                            <span style={{ paddingLeft: '0.5rem' }}>{status}</span>
                          </>
                          )
                        : (
                          <div className='text-truncate overflow-hidden text-nowrap'>
                            <span>Verify</span>
                            <span className='ml-1 text-nowrap'>
                          {currentFilename}
                        </span>
                          </div>
                          )}
                    </div>
                  </>
                  )}
            </div>
          </div>
        </button>
      </Container>
    )
  }

  return (
    <div>
      {verifications.map((verification) => {
        return verificationCard(
          verification.validation,
          verification.isLoading,
          verification.onClick
        )
      })}
    </div>
  )
}

export default Verification
