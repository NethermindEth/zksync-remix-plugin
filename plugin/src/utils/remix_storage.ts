import {ContractFile} from '@/types/contracts'
import {RemixClient} from '@/PluginClient'

export const getAllContractFiles = async (
    remixClient: RemixClient,
    workspacePath: string,
    dirPath = ''
): Promise<ContractFile[]> => {
    const files = [] as ContractFile[]
    const pathFiles = await remixClient.fileManager.readdir(`${workspacePath}/${dirPath}`)
    for (const [path, entry] of Object.entries<any>(pathFiles)) {
        if (entry.isDirectory) {
            const deps = await getAllContractFiles(remixClient, workspacePath, path)
            for (const dep of deps) files.push(dep)
            continue
        }

        const content = await remixClient.fileManager.readFile(path)

        if (!path.endsWith('.sol')) continue

        files.push({
            file_name: path,
            file_content: content,
            // we need to keep it, to persist the json file format
            is_contract: true
        })
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
        .filter(({file_name, is_contract}) => is_contract && !file_name.startsWith('contracts/'))
        .map(({file_name}) => file_name.split('/').pop())
}
