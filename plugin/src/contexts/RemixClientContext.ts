import { PluginClient } from '@remixproject/plugin'
import { createClient } from '@remixproject/plugin-webview'
import { createContext } from 'react'
import {WalletConnectRemixClient} from '../services/WalletConnectRemixClient'

//const remixClient = createClient(new PluginClient())
const remixClient = new WalletConnectRemixClient()

const RemixClientContext = createContext(remixClient)

export { RemixClientContext, remixClient }
