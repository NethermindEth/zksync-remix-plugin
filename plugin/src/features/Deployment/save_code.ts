import { apiUrl } from '../../utils/network'

export async function saveCode (
  solidityVersion: string,
  hashDir: string,
  currentFilePath: string,
  currentFileContent: string
): Promise<void> {
  const response = await fetch(
    `${apiUrl}/save_code/${solidityVersion}/${hashDir}/${currentFilePath}`,
    {
      method: 'POST',
      body: currentFileContent,
      redirect: 'follow',
      headers: {
        'Content-Type': 'application/octet-stream'
      }
    }
  )

  if (!response.ok) {
    throw new Error('Could not reach solidity verification server')
  }
}
