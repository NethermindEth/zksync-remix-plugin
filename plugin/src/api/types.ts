interface CompilationConfig {
  version: string
  user_libraries: string[]
  target_path?: string
}

export interface CompilationRequest {
  id: string
  config: CompilationConfig
}

export interface VerifyConfig {
  zksolc_version: string
  solc_version?: string
  network: string
  contract_address: string
  inputs: string[]
  target_contract?: string
}

export interface VerificationRequest {
  id: string
  config: VerifyConfig
}

export interface GeneratePresignedUrlsRequest {
  files: string[]
}

export interface GeneratePresignedUrlsResponse {
  id: string
  presigned_urls: string[]
}

export interface TaskFailure {
  error_type: string
  message: string
}

export enum ArtifactType {
  Unknown = 'Unknown',
  Contract = 'Contract',
  Dbg = 'Dbg'
}

export interface ArtifactInfo {
  artifact_type: ArtifactType
  file_path: string
  presigned_url: string
}

export interface TaskSuccessCompile {
  artifacts_info: ArtifactInfo[]
}

export interface TaskSuccessVerify {
  message: string
}

export type TaskSuccess = { Compile: TaskSuccessCompile } | { Verify: TaskSuccessVerify }
export type TaskResult = { Success: TaskSuccess } | { Failure: TaskFailure }

export function tryIntoFailureFromResult(taskResult: TaskResult): TaskFailure | undefined {
  if ('Failure' in taskResult) {
    return taskResult.Failure
  }

  return undefined
}

export function tryIntoSuccessFromResult(taskResult: TaskResult): TaskSuccess | undefined {
  if ('Success' in taskResult) {
    return taskResult.Success
  }

  return undefined
}

export function tryIntoCompileFromSuccess(taskSuccess: TaskSuccess): TaskSuccessCompile | undefined {
  if ('Compile' in taskSuccess) {
    return taskSuccess.Compile
  }

  return undefined
}

export function tryIntoVerifyFromSuccess(taskSuccess: TaskSuccess): TaskSuccessVerify | undefined {
  if ('Verify' in taskSuccess) {
    return taskSuccess.Verify
  }

  return undefined
}
