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

interface DeployedContract extends Contract {
  address: string
  transactionHash: string
  env: EnvType
}

interface CompilationResult {
  status: string
  message: string
  file_content: SolFile[]
}

interface VerificationResult {
  status: string
  message: string
}

interface SolFile {
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

interface Abi extends Array<AbiElement> {
}

type Output = Input

export type {
  Abi,
  AbiElement,
  Contract,
  Input,
  Output,
  DeployedContract,
  CompilationResult,
  VerificationResult,
  SolFile
}
