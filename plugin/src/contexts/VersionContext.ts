import type React from 'react'
import { createContext } from 'react'

const VersionContext = createContext({
  solidityVersion: '' as string,
  setSolidityVersion: ((_: string) => {
  }) as React.Dispatch<React.SetStateAction<string>>,
  versions: [] as string[],
  setVersions: ((_: string[]) => {
  }) as React.Dispatch<React.SetStateAction<string[]>>
})

export default VersionContext
