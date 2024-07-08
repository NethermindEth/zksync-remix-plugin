import { ContractFile } from '@/types/contracts'
import { RemixClient } from '@/PluginClient'

export const getAllContractFiles = async (remixClient: RemixClient, path: string): Promise<ContractFile[]> => {
  const files = [] as ContractFile[]
  const pathFiles = await remixClient.fileManager.readdir(`${path}/`)
  for (const [name, entry] of Object.entries<any>(pathFiles)) {
    if (entry.isDirectory) {
      const deps = await getAllContractFiles(remixClient, `${path}/${name}`)
      for (const dep of deps) files.push(dep)
    } else {
      const content = await remixClient.fileManager.readFile(name)
      files.push({
        file_name: name,
        file_content: content,
        is_contract: name.endsWith('.sol')
      })
    }
  }
  return files
}

export const getContractTargetPath = (contractFilePath: string) => {
  const parts = contractFilePath.split('/')
  if (parts.length === 1) {
    return './'
  } else {
    parts.pop()
    return './' + parts.join('/')
  }
}

export const findFilesNotInContracts = (allContracts: ContractFile[]) => {
  return allContracts
    .filter(({ file_name, is_contract }) => is_contract && !file_name.startsWith('contracts/'))
    .map(({ file_name }) => file_name.split('/').pop())
}
