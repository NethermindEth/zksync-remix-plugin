import React, { useContext, useEffect, useState } from 'react'
import { PluginClient } from '@remixproject/plugin'
import { createClient } from '@remixproject/plugin-webview'
import './App.css'
import Plugin from './features/Plugin'
import { RemixClientContext, remixClient } from './contexts/RemixClientContext'
import Loader from './ui_components/CircularLoader'
import FullScreenOverlay from './ui_components/FullScreenOverlay'
import { fetchGitHubFilesRecursively } from './utils/initial_scarb_codes'
import type {EthereumClient} from '@web3modal/ethereum'
import {WalletConnectUI} from './components/Wallet/walletConnectUI'
import {WalletConnectRemixClient} from './services/WalletConnectRemixClient'

//const remixClient = createClient(new PluginClient())


// //const remixClient = createClient(new PluginClient())
// const remixClient = createClient(new PluginClient())


// 3. Create modal
const App: React.FC = () => {
  const [pluginLoaded, setPluginLoaded] = useState<boolean>(false)
  const [ethereumClient, setEthereumClient] = useState<EthereumClient>(null)
  const [wagmiConfig, setWagmiConfig] = useState(null)
  const [theme, setTheme] = useState<string>('dark')
  // const remixClient = useContext(RemixClientContext)

  useEffect(() => {
    ;(async () => {
      await remixClient.initClient()
      remixClient.internalEvents.on('themeChanged', (theme: string) => {
        setTheme(theme)
      })

      setWagmiConfig(remixClient.wagmiConfig)
      setEthereumClient(remixClient.ethereumClient)
    })()
  }, [])
  


    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    // const id = setTimeout(async (): Promise<void> => {
    //   await remixClient.onload(() => {
    //     setPluginLoaded(true)
    //     // eslint-disable-next-line @typescript-eslint/no-misused-promises
    //     setTimeout(async () => {
    //       const workspaces = await remixClient.createdInternalClient.filePanel.getWorkspaces()

    //       const workspaceLets: Array<{ name: string, isGitRepo: boolean }> =
    //         JSON.parse(JSON.stringify(workspaces))

    //       if (
    //         !workspaceLets.some(
    //           (workspaceLet) => workspaceLet.name === 'cairo_scarb_sample'
    //         )
    //       ) {
    //         await remixClient.createdInternalClient.filePanel.createWorkspace(
    //           'cairo_scarb_sample',
    //           true
    //         )
    //         try {
    //           await remixClient.createdInternalClient.fileManager.mkdir('hello_world')
    //         } catch (e) {
    //           console.log(e)
    //         }
    //         const exampleRepo = await fetchGitHubFilesRecursively(
    //           'software-mansion/scarb',
    //           'examples/starknet_multiple_contracts'
    //         )

    //         try {
    //           for (const file of exampleRepo) {
    //             const filePath = file?.path
    //               .replace('examples/starknet_multiple_contracts/', '')
    //               .replace('examples/starknet_multiple_contracts', '') ?? ''

    //             let fileContent: string = file?.content ?? ''

    //             console.log('kljkljl', file?.fileName)

    //             if (file != null && file.fileName === 'Scarb.toml') {
    //               fileContent = fileContent.concat('\ncasm = true')
    //             }

    //             // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
    //             await remixClient.createdInternalClient.fileManager.writeFile(
    //               `hello_world/${
    //                 // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
    //                 filePath
    //               // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
    //               }/${file?.fileName}`,
    //               fileContent
    //             )
    //           }
    //         } catch (e) {
    //           if (e instanceof Error) {
    //             await remixClient.call('notification' as any, 'alert', {
    //               id: 'starknetRemixPluginAlert',
    //               title: 'Please check the write file permission',
    //               message: e.message + '\n' + 'Did you provide the write file permission?'
    //             })
    //           }
    //           console.log(e)
    //         }
    //       }
    //     })
    //   })
    // }, 1)
  //   return () => {
  //     clearInterval(id)
  //   }
  // })

  return (
    <div className="App">
      <h4 className="mt-1">WalletConnect</h4>
      {ethereumClient && wagmiConfig  != null && <WalletConnectUI wagmiConfig={wagmiConfig} ethereumClient={ethereumClient} theme={theme} />}
    </div>
  )
}

export default App
