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

export const appendContractPrefix = (contractFiles: ContractFile[]): ContractFile[] => {
  return contractFiles.map((contractFile) => {
    if (contractFile.file_name.endsWith('.sol') && !contractFile.file_name.startsWith('contracts/')) {
      contractFile.file_name = `contracts/${contractFile.file_name}`
    }
    return contractFile
  })
}
