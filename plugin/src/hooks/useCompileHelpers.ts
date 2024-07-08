import {
  CompilationType,
  compileErrorMessagesAtom,
  contractsAtom,
  selectedContractAtom,
  solidityVersionAtom
} from '@/atoms'
import { currentFilenameAtom, currentWorkspacePathAtom, remixClientAtom } from '@/stores/remixClient'
import { CompilationResult, Contract } from '@/types/contracts'
import { artifactFolder } from '@/utils/utils'
import { useAtomValue, useSetAtom } from 'jotai'

export const useCompileHelpers = () => {
  const remixClient = useAtomValue(remixClientAtom)
  const setCompileErrorMessages = useSetAtom(compileErrorMessagesAtom)
  const currentWorkspacePath = useAtomValue(currentWorkspacePathAtom)
  const solidityVersion = useAtomValue(solidityVersionAtom)
  const currentFilename = useAtomValue(currentFilenameAtom)
  const setContracts = useSetAtom(contractsAtom)
  const setSelectedContract = useSetAtom(selectedContractAtom)

  const getDefaultWorkspaceContents: any = () => ({
    config: {
      version: solidityVersion,
      user_libraries: []
    },
    contracts: [] as Array<{ file_name: string; file_content: string; is_contract: boolean }>
  })

  const emitErrorToRemix = async (compileResult: CompilationResult) => {
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
    const lastLine = compileResult.message.trim().split('\n').pop()?.trim()

    remixClient.emit('statusChanged', {
      key: 'failed',
      type: 'error',
      title: (lastLine ?? '').startsWith('Error') ? lastLine : 'Compilation Failed'
    })
    setCompileErrorMessages(errorLets)
    throw new Error('Solidity Compilation Failed, logs can be read in the terminal log')
  }

  const writeResultsToArtifacts = async (compileResult: CompilationResult) => {
    const artifacts: string[] = []
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
  }

  const setCompiledContracts = (compileResult: CompilationResult, compilationType: CompilationType) => {
    const contractsToAdd: Contract[] = []
    if (compilationType === 'PROJECT') {
      for (const file of compileResult.file_content) {
        if (!file.is_contract || !file.file_name.startsWith('contracts/')) continue

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
  }

  return {
    emitErrorToRemix,
    writeResultsToArtifacts,
    getDefaultWorkspaceContents,
    setCompiledContracts
  }
}
