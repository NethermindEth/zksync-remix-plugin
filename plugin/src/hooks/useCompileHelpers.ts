import {
  CompilationType,
  compileErrorMessagesAtom,
  contractsAtom,
  selectedContractAtom,
  solidityVersionAtom
} from '@/atoms'
import { currentFilenameAtom, currentWorkspacePathAtom, remixClientAtom } from '@/stores/remixClient'
import { CompiledArtifact, Contract } from '@/types/contracts'
import { artifactFolder } from '@/utils/utils'
import { useAtomValue, useSetAtom } from 'jotai'
import { CompilationRequest, TaskFailure } from '@/api/types'

export const useCompileHelpers = () => {
  const remixClient = useAtomValue(remixClientAtom)
  const setCompileErrorMessages = useSetAtom(compileErrorMessagesAtom)
  const currentWorkspacePath = useAtomValue(currentWorkspacePathAtom)
  const solidityVersion = useAtomValue(solidityVersionAtom)
  const currentFilename = useAtomValue(currentFilenameAtom)
  const setContracts = useSetAtom(contractsAtom)
  const setSelectedContract = useSetAtom(selectedContractAtom)

  const getDefaultCompilationRequest = (id: string): CompilationRequest => ({
    id,
    config: {
      version: '1.4.1' || solidityVersion,
      user_libraries: []
    }
  })

  const emitErrorToRemix = async (taskFailure: TaskFailure) => {
    await remixClient.terminal.log({
      value: `${taskFailure.error_type}: ${taskFailure.message}`,
      type: 'error'
    })

    const errorLets = taskFailure.message.trim().split('\n')
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
    const lastLine = taskFailure.message.trim().split('\n').pop()?.trim()

    remixClient.emit('statusChanged', {
      key: 'failed',
      type: 'error',
      title: (lastLine ?? '').startsWith('Error') ? lastLine : 'Compilation Failed'
    })
    setCompileErrorMessages(errorLets)
    throw new Error('Solidity Compilation Failed, logs can be read in the terminal log')
  }

  const writeResultsToArtifacts = async (artifacts: CompiledArtifact[]) => {
    for (const artifact of artifacts) {
      const artifactsPath = `${artifactFolder(currentWorkspacePath)}/${artifact.file_path}`

      try {
        await remixClient.call('fileManager', 'writeFile', artifactsPath, artifact.file_content)
        remixClient.emit('statusChanged', {
          key: 'succeed',
          type: 'info',
          title: 'Saved artifacts'
        })
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
      }
    }
  }

  const setCompiledContracts = (compilationArtifacts: CompiledArtifact[], compilationType: CompilationType) => {
    const contractsToAdd: Contract[] = []
    if (compilationType === 'PROJECT') {
      for (const artifact of compilationArtifacts) {
        if (!artifact.is_contract || !artifact.file_path.startsWith('contracts/')) continue

        const contract = JSON.parse(artifact.file_content) as Contract
        contractsToAdd.push(contract)
      }
    } else {
      for (const artifact of compilationArtifacts) {
        if (!artifact.is_contract || !artifact.file_path.includes(currentFilename)) continue

        const contract = JSON.parse(artifact.file_content) as Contract
        contractsToAdd.push(contract)
      }
    }

    setContracts(contractsToAdd)
    setSelectedContract(contractsToAdd[0])
  }

  return {
    emitErrorToRemix,
    writeResultsToArtifacts,
    getDefaultCompilationRequest,
    setCompiledContracts
  }
}
