import { atom, createStore } from 'jotai'
import { getFileExtension, getFileNameFromPath } from '@/utils/utils'
import { remixClient, RemixClient } from '@/PluginClient'
import { Workspace } from '@/types/plugin'

const WORKSPACE_ROOT = '.workspaces'

const remixClientAtom = atom({} as unknown as RemixClient)
const noFileSelectedAtom = atom(false)
const isValidSolidityAtom = atom(false)
const currentFilepathAtom = atom('')
const currentFilenameAtom = atom((get) => {
  const currentFilePath = get(currentFilepathAtom)
  return getFileNameFromPath(currentFilePath)
})

const currentWorkspacePathAtom = atom('')
const isLoadedAtom = atom(false)

const remixClientStore = createStore()

async function handleStatusChange(): Promise<void> {
  const isValidSolidity = remixClientStore.get(isValidSolidityAtom)
  const currentFilename = remixClientStore.get(currentFilenameAtom)
  if (isValidSolidity) {
    remixClient.emit('statusChanged', {
      key: 'succeed',
      type: 'info',
      title: 'Current file: ' + currentFilename
    })
  } else {
    remixClient.emit('statusChanged', {
      key: 'failed',
      type: 'warning',
      title: 'Please open a solidity file to compile'
    })
  }
}

async function initializeRemixClient(): Promise<RemixClient> {
  await remixClient.onload()

  const currWorkspace = await remixClient.filePanel.getCurrentWorkspace()
  remixClientStore.set(currentWorkspacePathAtom, currWorkspace.absolutePath)

  remixClient.on('fileManager', 'currentFileChanged', async (currentFileChanged: any) => {
    const filename = getFileNameFromPath(currentFileChanged)
    const currentFileExtension = getFileExtension(filename)
    const isValidSolidity = currentFileExtension === 'sol'
    remixClientStore.set(isValidSolidityAtom, isValidSolidity)
    remixClientStore.set(currentFilepathAtom, currentFileChanged)
    remixClientStore.set(noFileSelectedAtom, false)

    await handleStatusChange()
  })

  remixClient.on('fileManager', 'noFileSelected', async () => {
    remixClientStore.set(noFileSelectedAtom, true)
    remixClientStore.set(currentFilepathAtom, '')
    remixClientStore.set(isValidSolidityAtom, false)

    await handleStatusChange()
  })

  remixClient.on('filePanel', 'workspaceCreated', async (workspace) => {
    remixClientStore.set(noFileSelectedAtom, true)
    remixClientStore.set(currentFilepathAtom, '')
    remixClientStore.set(isValidSolidityAtom, false)
    remixClientStore.set(currentWorkspacePathAtom, `${WORKSPACE_ROOT}/${workspace as string}`)
  })

  remixClient.on('filePanel', 'setWorkspace', async (valueRaw) => {
    const workspace = valueRaw as Workspace

    remixClientStore.set(noFileSelectedAtom, true)
    remixClientStore.set(currentFilepathAtom, '')
    remixClientStore.set(isValidSolidityAtom, false)
    remixClientStore.set(currentWorkspacePathAtom, `${WORKSPACE_ROOT}/${workspace.name}`)
  })

  console.log('Remix client initialized')

  return remixClient
}

export {
  initializeRemixClient,
  remixClientStore,
  remixClientAtom,
  noFileSelectedAtom,
  isValidSolidityAtom,
  currentFilenameAtom,
  currentWorkspacePathAtom,
  isLoadedAtom,
  currentFilepathAtom
}
