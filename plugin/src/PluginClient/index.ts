import { PluginClient } from '@remixproject/plugin'
import axios from 'axios'
import { createClient } from '@remixproject/plugin-webview'
import { Workspace } from '@/types/plugin'

interface GithubRepoQuery {
  owner: string
  repo: string
  path: string
  ref?: string
}

enum EntryType {
  File = 'file',
  Dir = 'dir'
}

interface GithubEntry {
  name: string
  path: string
  type: EntryType
  url: string
  gitUrl: string
  htmlUrl: string
}

export class ZKSyncPluginClient extends PluginClient {
  methods = ['loadFromUrl', 'loadFromGithub']

  constructor() {
    super()
  }

  private static parseGithubUrl(candidateUrl: string): GithubRepoQuery {
    const url = new URL(candidateUrl)
    const [owner, repo, ...pathArray] = url.pathname.split('/').filter(Boolean)
    if (!owner || !repo) {
      throw Error(`Invalid github url: ${candidateUrl}`)
    }

    const path = pathArray.join('/')
    const ref = url.searchParams.get('ref') || undefined
    return { owner, repo, path, ref }
  }

  private static extractEntriesFromData(rawData: any): GithubEntry[] {
    return rawData.map((rawEl: any) => {
      if (!rawEl.name || !rawEl.path || !rawEl.type || !rawEl.url || !rawEl.git_url || !rawEl.html_url) {
        throw Error('Invalid github data entry format')
      }

      return {
        name: rawEl.name,
        path: rawEl.path,
        type: rawEl.type,
        url: rawEl.url,
        gitUrl: rawEl.git_url,
        htmlUrl: rawEl.html_url
      }
    })
  }

  async loadFromUrl(url: string): Promise<void> {
    // TODO(edwin): remove excessive logging after prod testing
    let query
    try {
      query = ZKSyncPluginClient.parseGithubUrl(url)
    } catch (error) {
      console.log(`Error parsing github url: ${url}, error:`, error)
      throw error
    }

    return await this.loadFromGithub(query.owner, query.repo, query.path, query.ref)
  }

  async loadFromGithub(owner: string, repo: string, path?: string, ref?: string): Promise<void> {
    if (!owner || !repo) {
      throw Error('owner and repo arguments are required.')
    }

    const query: GithubRepoQuery = { owner, repo, path: path || '', ref }
    const nameExistsAlready = await this.validateWorkspaceCreation(query)
    if (nameExistsAlready) {
      throw Error('Name already exists')
    }

    try {
      return await this.loadRepository(query)
    } catch (error) {
      console.log('Error loading repository with query:', query)
      throw error
    }
  }

  private async loadRepository(query: GithubRepoQuery): Promise<void> {
    const { owner, repo, path } = query

    // Create new workspace for repo
    await this.call('filePanel', 'createWorkspace', repo, true)

    try {
      const getApiUrl = (ref?: string) => {
        if (!ref) {
          return `https://api.github.com/repos/${owner}/${repo}/contents/${path}`
        }

        return `https://api.github.com/repos/${owner}/${repo}/contents/${path}?ref=${ref}`
      }

      const content = await axios.get(getApiUrl(query.ref))
      const entries = ZKSyncPluginClient.extractEntriesFromData(content.data)

      await this.loadRepositoryImpl(entries)
    } catch (error) {
      console.log('Error loading repository', error)
      throw error
    }
  }

  private async loadRepositoryImpl(entries: GithubEntry[]) {
    for (const entry of entries) {
      switch (entry.type) {
        case EntryType.Dir: {
          await this.call('fileManager', 'mkdir', entry.path)

          const content = await axios.get(entry.url)
          const entries = ZKSyncPluginClient.extractEntriesFromData(content.data)
          await this.loadRepositoryImpl(entries)
          break
        }

        case EntryType.File: {
          const content = await this.call('contentImport', 'resolve', entry.htmlUrl)
          await this.call('fileManager', 'writeFile', entry.path, content.content)
        }
      }
    }
  }

  private async validateWorkspaceCreation(query: GithubRepoQuery): Promise<boolean> {
    const rawWorkspaces = await this.call('filePanel', 'getWorkspaces')
    const workspaces: Array<Workspace> = JSON.parse(JSON.stringify(rawWorkspaces))

    // TODO(edwin): use owner ++ repo?
    const exist = workspaces.some((workspace) => workspace.name === query.repo)
    return exist
  }
}

export const remixClient = createClient(new ZKSyncPluginClient())
export type RemixClient = typeof remixClient
