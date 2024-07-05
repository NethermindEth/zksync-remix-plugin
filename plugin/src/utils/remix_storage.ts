import { ContractFile } from '@/types/contracts'
import { RemixClient } from '@/PluginClient'

export const getAllContractFiles = async (remixClient: RemixClient, path: string): Promise<ContractFile[]> => {
  const files = [] as ContractFile[]
  const pathFiles = await remixClient.fileManager.readdir(`${path}/`)
  for (const [name, entry] of Object.entries<any>(pathFiles)) {
    if (entry.isDirectory) {
      console.log('directory entry', entry, name)
      const deps = await getAllContractFiles(remixClient, `${path}/${name}`)
      console.log('directory deps', deps)
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

export const getContractTargetPath = (allContracts: ContractFile[], contractFileName: string) => {
  for (const { file_name } of allContracts) {
    if (file_name.includes(contractFileName)) {
      const parts = file_name.split('/')
      if (parts.length === 1) {
        return './'
      } else {
        parts.pop()
        return './' + parts.join('/')
      }
    }
  }

  return './'
}
