import type { EnvType } from './transaction'

interface Contract {
  contractName: string
  sourceName: string
  abi: Abi
  bytecode: string
  deployedBytecode: string
  linkReferences: Record<string, unknown>
  deployedLinkReferences: Record<string, unknown>
  factoryDeps: Record<string, unknown>
}

interface ContractFile {
  file_name: string
  file_content: string
  is_contract: boolean
}

interface DeployedContract extends Contract {
  address: string
  transactionHash: string
  env: EnvType
}

interface CompilationResult {
  status: string
  message: string
  file_content: CompiledArtifact[]
}

interface VerificationResult {
  status: string
  message: string
}

interface CompiledArtifact {
  // Is a relative path to a file
  file_name: string
  file_content: string
  is_contract: boolean
}

interface Input {
  name: string
  type: string
  internalType?: string
}

interface AbiElement {
  inputs: Input[]
  name: string
  outputs: Output[]
  stateMutability: string
  type: string
}

type Abi = Array<AbiElement>

type Output = Input

export type {
  Abi,
  AbiElement,
  Contract,
  ContractFile,
  Input,
  Output,
  DeployedContract,
  CompilationResult,
  VerificationResult,
  CompiledArtifact
}
