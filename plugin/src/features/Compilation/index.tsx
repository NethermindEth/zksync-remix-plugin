import React from 'react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import { artifactFolder } from '@/utils/utils'
import Container from '../../ui_components/Container'
import { type AccordianTabs } from '../Plugin'
import { type CompilationResult, type Contract } from '../../types/contracts'
import { asyncGet } from '../../api/asyncRequests'
import { compilationAtom, isCompilingAtom, statusAtom } from '../../atoms/compilation'
import { solidityVersionAtom } from '../../atoms/version'
import { contractsAtom, selectedContractAtom } from '../../atoms/compiledContracts'
import {
  activeTomlPathAtom,
  currentFilenameAtom,
  isValidSolidityAtom,
  remixClientAtom,
  tomlPathsAtom
} from '../../stores/remixClient'
import { saveCode } from '../../api/saveCode'
import './styles.css'

interface CompilationProps {
  setAccordian: React.Dispatch<React.SetStateAction<AccordianTabs>>
}

export const Compilation: React.FC<CompilationProps> = ({ setAccordian }) => {
  const remixClient = useAtomValue(remixClientAtom)
  const isValidSolidity = useAtomValue(isValidSolidityAtom)
  const currentFilename = useAtomValue(currentFilenameAtom)
  const tomlPaths = useAtomValue(tomlPathsAtom)
  const activeTomlPath = useAtomValue(activeTomlPathAtom)

  const [contracts, setContracts] = useAtom(contractsAtom)
  const setSelectedContract = useSetAtom(selectedContractAtom)

  const { status, isCompiling, hashDir } = useAtomValue(compilationAtom)

  const setStatus = useSetAtom(statusAtom)
  const setIsCompiling = useSetAtom(isCompilingAtom)

  const solidityVersion = useAtomValue(solidityVersionAtom)

  const compilations = [
    {
      validation: isValidSolidity,
      isLoading: isCompiling,
      onClick: compile,
      id: 1
    }
  ]

  async function compile(): Promise<void> {
    setIsCompiling(true)
    setStatus('Compiling...')
    // clear current file annotations: inline syntax error reporting
    await remixClient.editor.clearAnnotations()
    try {
      setStatus('Getting solidity file path...')
      const currentFilePath = await remixClient.call('fileManager', 'getCurrentFile')

      setStatus('Getting solidity file content...')
      const currentFileContent = await remixClient.call('fileManager', 'readFile', currentFilePath)

      setStatus('Parsing solidity code...')
      await saveCode(solidityVersion, hashDir, currentFilePath, currentFileContent)

      setStatus('Compiling...')

      const response = await asyncGet(
        `compile-async/${solidityVersion}/${hashDir}/${currentFilePath}`,
        'compile-result'
      )

      if (!response.ok) throw new Error('Solidity Compilation Request Failed')
      else {
        await remixClient.call('notification' as any, 'toast', 'Solidity compilation request successful')
      }

      // get Json body from response
      const compileResult = JSON.parse(await response.text()) as CompilationResult

      if (compileResult.status !== 'Success') {
        setStatus('Reporting Errors...')
        await remixClient.terminal.log({
          value: compileResult.message,
          type: 'error'
        })

        const errorLets = compileResult.message.trim().split('\n')

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
        const lastLine = compileResult.message.trim().split('\n').pop()?.trim()

        remixClient.emit('statusChanged', {
          key: 'failed',
          type: 'error',
          title: (lastLine ?? '').startsWith('Error') ? lastLine : 'Compilation Failed'
        })
        throw new Error('Solidity Compilation Failed, logs can be read in the terminal log')
      }

      const artifacts: string[] = []

      const contractsToAdd: Contract[] = []
      for (const file of compileResult.file_content) {
        const contract = JSON.parse(file.file_content) as Contract
        contractsToAdd.push(contract)
      }

      setContracts([...contractsToAdd, ...contracts])
      setSelectedContract(contractsToAdd[0])

      for (const file of compileResult.file_content) {
        const artifactsPath = `${artifactFolder(currentFilePath)}/${file.file_name}`
        artifacts.push(artifactsPath)

        try {
          await remixClient.call('fileManager', 'writeFile', artifactsPath, file.file_content)
        } catch (e) {
          if (e instanceof Error) {
            await remixClient.call(
              'notification' as any,
              'toast',
              e.message + ' try deleting the files: ' + artifactsPath
            )
          }
          remixClient.emit('statusChanged', {
            key: 'succeed',
            type: 'warning',
            title: 'Failed to save artifacts'
          })
        } finally {
          remixClient.emit('statusChanged', {
            key: 'succeed',
            type: 'info',
            title: 'Saved artifacts'
          })
        }
      }

      setAccordian('deploy')
    } catch (e) {
      setStatus('failed')
      if (e instanceof Error) {
        await remixClient.call('notification' as any, 'alert', {
          id: 'zksyncRemixPluginAlert',
          title: 'Compilation Failed',
          message: e.message
        })
      }
      console.error(e)
    } finally {
      setIsCompiling(false)
    }
  }

  const compilationCard = (
    validation: boolean,
    isLoading: boolean,
    onClick: () => unknown,
    id: number
  ): React.ReactElement => {
    return (
      <Container key={id}>
        {activeTomlPath && tomlPaths?.length > 0 && (
          <div className="project-dropdown-wrapper d-flex flex-column mb-3">
            <div className="mx-auto">Compile a single Solidity file:</div>
          </div>
        )}
        <button
          className="btn btn-primary w-100 text-break remixui_disabled mb-1 mt-1 px-0"
          style={{
            cursor: `${!validation || !currentFilename ? 'not-allowed' : 'pointer'}`
          }}
          disabled={!validation || !currentFilename || isCompiling}
          aria-disabled={!validation || !currentFilename || isCompiling}
          onClick={onClick}
        >
          <div className="d-flex align-items-center justify-content-center">
            <div className="text-truncate overflow-hidden text-nowrap">
              {!validation ? (
                <span>Select a valid solidity file</span>
              ) : (
                <>
                  <div className="d-flex align-items-center justify-content-center">
                    {isLoading ? (
                      <>
                        <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true">
                          {' '}
                        </span>
                        <span style={{ paddingLeft: '0.5rem' }}>{status}</span>
                      </>
                    ) : (
                      <div className="text-truncate overflow-hidden text-nowrap">
                        <span>Compile</span>
                        <span className="ml-1 text-nowrap">{currentFilename}</span>
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
      {compilations.map((compilation) => {
        return compilationCard(compilation.validation, compilation.isLoading, compilation.onClick, compilation.id)
      })}
    </div>
  )
}
