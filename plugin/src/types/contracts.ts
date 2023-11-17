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
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(crate = "rocket::serde")]
// pub struct CompileResponse {
//   pub status: String,
//     pub message: String,
//     pub file_content: Vec<SolFile>,
// }
//
// #[derive(Debug, Deserialize, Serialize)]
// #[serde(crate = "rocket::serde")]
// pub struct SolFile {
//   pub file_name: String,
//     pub file_content: String,
// }
interface CompilationResult {
  status: string
  message: string
  file_content: SolFile[]
}

interface SolFile {
  file_name: string
  file_content: string
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
  SolFile
}
