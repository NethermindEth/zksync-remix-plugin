import { PluginClient } from '@remixproject/plugin'
import { createClient } from '@remixproject/plugin-webview'
import { atom, createStore } from 'jotai'
import { getFileExtension, getFileNameFromPath } from '../utils/utils'

const remixClientAtom = atom({} as unknown as RemixClient)
const noFileSelectedAtom = atom(false)
const isValidSolidityAtom = atom(false)
const currentFilenameAtom = atom('')
const currentWorkspacePathAtom = atom('')
const tomlPathsAtom = atom<string[]>([])
const activeTomlPathAtom = atom('')
const isLoadedAtom = atom(false)

const remixClientStore = createStore()

const remixClient = createClient(new PluginClient())
type RemixClient = typeof remixClient

async function getTomlPaths (workspacePath: string, currPath: string): Promise<string[]> {
  const resTomlPaths: string[] = []

  try {
    const allFiles = await remixClient.fileManager.readdir(`${workspacePath}/${currPath}`)
    // get keys of allFiles object
    const allFilesKeys = Object.keys(allFiles)
    // const get all values of allFiles object
    const allFilesValues = Object.values(allFiles)

    for (let i = 0; i < allFilesKeys.length; i++) {
      if (allFilesKeys[i].endsWith('Scarb.toml')) {
        resTomlPaths.push(currPath)
      }
      if (Object.values(allFilesValues[i])[0].length > 0) {
        const recTomlPaths = await getTomlPaths(workspacePath, allFilesKeys[i])
        resTomlPaths.push(...recTomlPaths)
      }
    }
  } catch (e) {
    console.error('getTomlPaths() error: ', e)
  }
  return resTomlPaths
}

async function handleTomlPathsChange (): Promise<void> {
  try {
    const allTomlPaths = await getTomlPaths(remixClientStore.get(currentWorkspacePathAtom), '')
    remixClientStore.set(tomlPathsAtom, allTomlPaths)
    const activeTomlPath = remixClientStore.get(activeTomlPathAtom)
    if (activeTomlPath === '' || activeTomlPath === undefined) {
      remixClientStore.set(activeTomlPathAtom, allTomlPaths[0])
    }
  } catch (err) {
    console.error('handleTomlPathsChange() error: ', err)
  }
}

async function initializeRemixClient (): Promise<void> {
  await remixClient.onload()

  const currWorkspace = await remixClient.filePanel.getCurrentWorkspace()
  remixClientStore.set(currentWorkspacePathAtom, currWorkspace.absolutePath)

  remixClient.on('fileManager', 'currentFileChanged', (currentFileChanged: any) => {
    const filename = getFileNameFromPath(currentFileChanged)
    const currentFileExtension = getFileExtension(filename)
    const isValidSolidity = currentFileExtension === 'sol'
    remixClientStore.set(isValidSolidityAtom, isValidSolidity)
    remixClientStore.set(currentFilenameAtom, filename)
    remixClientStore.set(noFileSelectedAtom, false)
  })

  remixClient.on('fileManager', 'noFileSelected', () => {
    remixClientStore.set(noFileSelectedAtom, true)
    remixClientStore.set(currentFilenameAtom, '')
    remixClientStore.set(isValidSolidityAtom, false)
  })

  remixClient.on('fileManager', 'fileAdded', async (_: any) => {
    await handleTomlPathsChange()
  })

  remixClient.on('fileManager', 'folderAdded', async (_: any) => {
    await handleTomlPathsChange()
  })

  remixClient.on('fileManager', 'fileRemoved', async (_: any) => {
    await handleTomlPathsChange()
  })

  remixClient.on('filePanel', 'workspaceCreated', async (_: any) => {
    await handleTomlPathsChange()
  })

  remixClient.on('filePanel', 'workspaceRenamed', async (_: any) => {
    await handleTomlPathsChange()
  })

  remixClientStore.set(isLoadedAtom, true)
  console.log('Remix client initialized')
}

void (async () => { await initializeRemixClient() })()
remixClientStore.set(remixClientAtom, remixClient)

export {
  remixClientStore,
  remixClientAtom,
  noFileSelectedAtom,
  isValidSolidityAtom,
  currentFilenameAtom,
  currentWorkspacePathAtom,
  tomlPathsAtom,
  activeTomlPathAtom,
  isLoadedAtom
}
