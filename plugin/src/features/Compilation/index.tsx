import React, { useEffect, useState } from 'react'
import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import Container from '@/ui_components/Container'
import { type AccordianTabs } from '@/types/common'
import { type CompilationResult } from '@/types/contracts'
import { asyncPost } from '@/api/asyncRequests'
import {
  compilationAtom,
  isCompilingAtom,
  compileStatusAtom,
  compileErrorMessagesAtom,
  deployStatusAtom,
  compilationTypeAtom,
  CompilationType
} from '@/atoms'
import {
  currentFilenameAtom,
  currentFilepathAtom,
  currentWorkspacePathAtom,
  isValidSolidityAtom,
  remixClientAtom
} from '@/stores/remixClient'
import './styles.css'
import { findFilesNotInContracts, getAllContractFiles, getContractTargetPath } from '@/utils/remix_storage'
import { Tooltip } from '@/ui_components'
import { FILES_NOT_IN_CONTRACTS_MESSAGE } from '@/utils/constants'
import { useCompileHelpers } from '@/hooks/useCompileHelpers'

interface CompilationProps {
  setAccordian: React.Dispatch<React.SetStateAction<AccordianTabs>>
}

export const Compilation = ({ setAccordian }: CompilationProps) => {
  const remixClient = useAtomValue(remixClientAtom)
  const isValidSolidity = useAtomValue(isValidSolidityAtom)
  const currentFilename = useAtomValue(currentFilenameAtom)
  const currentFilepath = useAtomValue(currentFilepathAtom)
  const currentWorkspacePath = useAtomValue(currentWorkspacePathAtom)
  const { status, isCompiling } = useAtomValue(compilationAtom)

  const setDeployStatus = useSetAtom(deployStatusAtom)

  const setCompileStatus = useSetAtom(compileStatusAtom)
  const setIsCompiling = useSetAtom(isCompilingAtom)
  const setCompileErrorMessages = useSetAtom(compileErrorMessagesAtom)

  const [compilationType, setCompilationType] = useAtom(compilationTypeAtom)

  const [isContractsFolderAvailable, setIsContractsFolderAvailable] = useState(true)
  const { emitErrorToRemix, writeResultsToArtifacts, getDefaultWorkspaceContents, setCompiledContracts } =
    useCompileHelpers()

  useEffect(() => {
    remixClient.fileManager
      .readdir(`${currentWorkspacePath}/`)
      .then((workspaceFiles: any) => {
        setIsContractsFolderAvailable(!!workspaceFiles?.contracts?.isDirectory)
      })
      .catch((error) => {
        console.error(`Failed to read current workspace ${error.message}`)
      })
  }, [currentWorkspacePath, remixClient])

  async function handleCompile({ type }: { type: CompilationType }): Promise<void> {
    const workspaceContents = getDefaultWorkspaceContents()

    setCompilationType(type)
    setIsCompiling(true)
    setDeployStatus('IDLE')
    setCompileStatus('Compiling...')
    setCompileErrorMessages([])
    await remixClient.editor.clearAnnotations()
    try {
      const allContractFiles = await getAllContractFiles(remixClient, currentWorkspacePath)
      workspaceContents.contracts = allContractFiles

      if (type === 'SINGLE_FILE') {
        const targetPath = getContractTargetPath(currentFilepath)
        workspaceContents.target_path = targetPath
      } else {
        const filesNotInContractsFolder = findFilesNotInContracts(allContractFiles)
        if (filesNotInContractsFolder.length > 0) {
          await remixClient.terminal.log({
            value: `${FILES_NOT_IN_CONTRACTS_MESSAGE} ${filesNotInContractsFolder.join('  ')}`,
            type: 'warn'
          })
        }
      }

      const response = await asyncPost('compile-async', 'compile-result', workspaceContents)

      if (!response.ok) throw new Error('Solidity Compilation Request Failed')
      else {
        await remixClient.call('notification' as any, 'toast', 'Solidity compilation request successful')
      }

      const compileResult = JSON.parse(await response.text()) as CompilationResult

      if (compileResult.status !== 'Success') {
        await emitErrorToRemix(compileResult)
      }
      console.log('compile result', compileResult)
      setCompiledContracts(compileResult, type)
      writeResultsToArtifacts(compileResult)

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
    }
  }

  return (
    <Container className="flex flex-column justify-content-center">
      <button
        className="btn btn-secondary d-block text-break mb-1 text-center"
        onClick={() => handleCompile({ type: 'PROJECT' })}
        disabled={isCompiling || !isContractsFolderAvailable}
      >
        {isCompiling && compilationType === 'PROJECT' ? (
          <>
            <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true">
              {' '}
            </span>
            <span style={{ paddingLeft: '0.5rem' }}>{status}</span>
          </>
        ) : isContractsFolderAvailable ? (
          <div className="text-truncate overflow-hidden text-nowrap">Compile Project</div>
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
        className="btn btn-primary d-block text-break mb-1 mt-2"
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
              {isCompiling && compilationType === 'SINGLE_FILE' ? (
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
