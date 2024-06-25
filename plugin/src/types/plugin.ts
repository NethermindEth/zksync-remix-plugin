export interface Workspace {
  hasGitSubmodules: boolean
  isGitRepo: boolean
  name: string
}

export interface WorkspaceCreated {
  name: string
  isLocalhost: boolean
}
