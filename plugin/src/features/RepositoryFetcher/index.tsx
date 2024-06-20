import React, { useState } from 'react'
import Container from '@/ui_components/Container'
import InputField from '@/components/InputField'
import axios from 'axios'
import { RemixClient, remixClientAtom } from '@/stores/remixClient'
import { useAtomValue } from 'jotai'

interface GithubRepoQuery {
  owner: string
  repo: string
  path: string[]
}

interface Workspace {
  hasGitSubmodules: boolean
  isGitRepo: boolean
  name: string
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

function extractEntriesFromData(rawData: any): GithubEntry[] {
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

async function fetchRepo(client: RemixClient, query: GithubRepoQuery): Promise<void> {
  // TODO(edwin): verify path empty?
  const { owner, repo, path } = query

  // Create new workspace for repo
  await client.filePanel.createWorkspace(query.repo, true)

  // sets to just created one
  // console.log(await client.filePanel.getCurrentWorkspace())

  try {
    const apiUrl = `https://api.github.com/repos/${owner}/${repo}/contents/${path}`
    const content = await axios.get(apiUrl)
    console.log(content.data)
    // Exception
    const entries = extractEntriesFromData(content.data)
    // Exception
    await addRepoData(client, entries)
  } catch (err) {
    console.log('Some error', err)
  }
}

async function addRepoData(client: RemixClient, entries: GithubEntry[]) {
  for (const entry of entries) {
    switch (entry.type) {
      case EntryType.Dir: {
        await client.fileManager.mkdir(entry.path)

        const content = await axios.get(entry.url)
        const entries = extractEntriesFromData(content.data)
        await addRepoData(client, entries)
        break
      }

      case EntryType.File: {
        const content = await client.contentImport.resolve(entry.htmlUrl)
        console.log('EntryType.File', content.content)

        await client.fileManager.writeFile(entry.path, content.content)
      }
    }
  }
}

function parseGithubUrl(candidateUrl: string): GithubRepoQuery {
  const url = new URL(candidateUrl)
  const [owner, repo, ...path] = url.pathname.split('/').filter(Boolean)
  if (!owner || !repo) {
    throw Error('Invalid github url')
  }

  return { owner, repo, path }
}

async function validateWorkspaceCreation(client: RemixClient, query: GithubRepoQuery): Promise<boolean> {
  const rawWorkspaces = await client.filePanel.getWorkspaces()
  const workspaces: Array<Workspace> = JSON.parse(JSON.stringify(rawWorkspaces))
  const exist = workspaces.some((workspace) => workspace.name === query.repo)

  return exist
}

export const RepositoryFetcher: React.FC = () => {
  const remixClient = useAtomValue(remixClientAtom)

  const [url, setUrl] = useState<string>('')
  const [fetching, setFetching] = useState<boolean>(false)

  async function addRepository(): Promise<void> {
    let query
    try {
      query = parseGithubUrl(url)
    } catch (err) {
      // TODO(edwin): set fancy UI error
      console.log(err)
      return
    }

    const nameExistsAlready = await validateWorkspaceCreation(remixClient, query)
    if (nameExistsAlready) {
      // TODO(edwin): set fancy UI error
      console.log('Name already exists')
      return
    }

    await fetchRepo(remixClient, query)
  }

  return (
    <div>
      <Container>
        <label className="label-repo-fetcher">Repository URL</label>
        <InputField
          index={0}
          value={url}
          placeholder={'Repository (URL)'}
          onChange={(_, newValue) => {
            setUrl(newValue)
          }}
        />
        <button
          className={'fetch-repo'}
          disabled={url.length === 0 || fetching}
          onClick={() => {
            setFetching(true)
            addRepository()
              .then(() => setFetching(false))
              .catch((err) => {
                setFetching(false)
                console.log(err)
              })
          }}
        >
          Load
        </button>
      </Container>
    </div>
  )
}
