import React, { useEffect, useState } from 'react'
import { useAtomValue, useSetAtom } from 'jotai'
import { artifactFolder } from '@/utils/utils'
import Container from '@/ui_components/Container'
import { type AccordianTabs } from '@/types/common'
import { type CompilationResult, type Contract } from '@/types/contracts'
import { asyncPost } from '@/api/asyncRequests'
import {
  compilationAtom,
  isCompilingAtom,
  compileStatusAtom,
  solidityVersionAtom,
  contractsAtom,
  selectedContractAtom,
  compileErrorMessagesAtom,
  deployStatusAtom
} from '@/atoms'
import {
  currentFilenameAtom,
  currentWorkspacePathAtom,
  isValidSolidityAtom,
  remixClientAtom
} from '@/stores/remixClient'
import './styles.css'
import { getAllContractFiles, getContractTargetPath } from '@/utils/remix_storage'
import { Tooltip } from '@/ui_components'

interface CompilationProps {
  setAccordian: React.Dispatch<React.SetStateAction<AccordianTabs>>
}

export const Compilation = ({ setAccordian }: CompilationProps) => {
  const remixClient = useAtomValue(remixClientAtom)
  const isValidSolidity = useAtomValue(isValidSolidityAtom)
  const currentFilename = useAtomValue(currentFilenameAtom)
  const currentWorkspacePath = useAtomValue(currentWorkspacePathAtom)

  const setContracts = useSetAtom(contractsAtom)
  const setSelectedContract = useSetAtom(selectedContractAtom)
  const setDeployStatus = useSetAtom(deployStatusAtom)

  const { status, isCompiling } = useAtomValue(compilationAtom)

  const setCompileStatus = useSetAtom(compileStatusAtom)
  const setIsCompiling = useSetAtom(isCompilingAtom)
  const [isCompilingProject, setIsCompilingProject] = useState(false)
  const setCompileErrorMessages = useSetAtom(compileErrorMessagesAtom)
  const [isContractsFolderAvailable, setIsContractsFolderAvailable] = useState(true)

  const solidityVersion = useAtomValue(solidityVersionAtom)

  useEffect(() => {
    remixClient.fileManager.readdir(`${currentWorkspacePath}/`).then((workspaceFiles: any) => {
      if (!workspaceFiles || !workspaceFiles['contracts'] || workspaceFiles['contracts']['isDirectory'] === false) {
        setIsContractsFolderAvailable(false)
      } else {
        setIsContractsFolderAvailable(true)
      }
    })
  }, [currentWorkspacePath, remixClient])

  async function handleCompile({ type }: { type: 'PROJECT' | 'SINGLE_FILE' }): Promise<void> {
    const workspaceContents: any = {
      config: {
        version: solidityVersion,
        user_libraries: []
      },
      contracts: [] as Array<{ file_name: string; file_content: string; is_contract: boolean }>
    }
    setDeployStatus('IDLE')
    setCompileStatus('Compiling...')
    setCompileErrorMessages([])
    // clear current file annotations: inline syntax error reporting
    await remixClient.editor.clearAnnotations()
    if (type === 'PROJECT') {
      setIsCompilingProject(true)
    } else {
      setIsCompiling(true)
    }
    try {
      const workspaceFiles = await remixClient.fileManager.readdir(`${currentWorkspacePath}/`)
      console.log(`workspaceFiles: ${JSON.stringify(workspaceFiles)}`)

      setCompileStatus('Compiling...')
      //TODO: here change the logic to only send single file in case of signle file compilation
      const allContractFiles = await getAllContractFiles(remixClient, currentWorkspacePath)
      workspaceContents.contracts = allContractFiles

      if (type === 'SINGLE_FILE') {
        const targetPath = getContractTargetPath(allContractFiles, currentFilename)
        workspaceContents.target_path = targetPath
      }
      const response = await asyncPost('compile-async', 'compile-result', workspaceContents)

      if (!response.ok) throw new Error('Solidity Compilation Request Failed')
      else {
        await remixClient.call('notification' as any, 'toast', 'Solidity compilation request successful')
      }

      const compileResult = JSON.parse(await response.text()) as CompilationResult

      if (compileResult.status !== 'Success') {
        setCompileStatus('Reporting Errors...')
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
        console.log('error lets array', errorLetsArray)
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
        setCompileErrorMessages(errorLets)
        throw new Error('Solidity Compilation Failed, logs can be read in the terminal log')
      }

      const artifacts: string[] = []

      const contractsToAdd: Contract[] = []
      if (type === 'PROJECT') {
        for (const file of compileResult.file_content) {
          if (!file.is_contract) continue
          const contract = JSON.parse(file.file_content) as Contract
          contractsToAdd.push(contract)
        }
      } else {
        for (const file of compileResult.file_content) {
          if (!file.is_contract || !file.file_name.includes(currentFilename)) continue
          const contract = JSON.parse(file.file_content) as Contract
          contractsToAdd.push(contract)
        }
      }
      setContracts(contractsToAdd)
      setSelectedContract(contractsToAdd[0])

      for (const file of compileResult.file_content) {
        const artifactsPath = `${artifactFolder(currentWorkspacePath)}/${file.file_name}`
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
      setCompileStatus('done')
      setAccordian('deploy')
    } catch (e) {
      setCompileStatus('failed')
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
      setIsCompilingProject(false)
    }
  }

  return (
    <Container>
      {/* <div className="align-center d-flex justify-content-center">
        Only files in the contracts folder can be compiled.
      </div> */}

      <button
        className="btn btn-warning w-100 text-break remixui_disabled mb-1 mt-1 px-2"
        onClick={() => handleCompile({ type: 'PROJECT' })}
        disabled={isCompilingProject || !isContractsFolderAvailable}
      >
        {isCompilingProject ? (
          <>
            <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true">
              {' '}
            </span>
            <span style={{ paddingLeft: '0.5rem' }}>{status}</span>
          </>
        ) : isContractsFolderAvailable ? (
          <span> Compile Project</span>
        ) : (
          <Tooltip
            icon={
              <div className="text-truncate overflow-hidden text-nowrap">
                <span>Compile Project</span>
              </div>
            }
            content={
              <div className="p-2 overflow-visible text-center text-wrap">
                Contracts folder not found in the workspace
              </div>
            }
            side="bottom"
            avoidCollisions={false}
          />
        )}
      </button>

      <button
        className="btn btn-primary w-100 text-break remixui_disabled mb-1 mt-1 px-2"
        style={{
          cursor: `${!isValidSolidity || !currentFilename ? 'not-allowed' : 'pointer'}`
        }}
        disabled={!isValidSolidity || !currentFilename || isCompiling}
        aria-disabled={!isValidSolidity || !currentFilename || isCompiling}
        onClick={() => handleCompile({ type: 'SINGLE_FILE' })}
      >
        <div className="d-flex align-items-center justify-content-center w-100">
          {!isValidSolidity ? (
            <div>Select a valid solidity file</div>
          ) : (
            <div className="d-flex align-items-center justify-content-center w-100">
              {isCompiling ? (
                <>
                  <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true">
                    {' '}
                  </span>
                  <span style={{ paddingLeft: '0.5rem' }}>{status}</span>
                </>
              ) : (
                <Tooltip
                  icon={
                    <div className="text-truncate overflow-hidden text-nowrap">
                      <span>Compile</span>
                      <span className="ml-1 text-nowrap">{currentFilename}</span>
                    </div>
                  }
                  content={<div className="p-2 overflow-visible text-center">{currentFilename}</div>}
                  side="bottom"
                  avoidCollisions={false}
                />
              )}
            </div>
          )}
        </div>
      </button>
    </Container>
  )
}
