import React, { type ReactNode, useEffect, useRef, useState } from 'react'
import { PluginClient } from '@remixproject/plugin'
import { type PluginApi, type Api } from '@remixproject/plugin-utils'
import { type IRemixApi } from '@remixproject/plugin-api'
import { createClient } from '@remixproject/plugin-webview'

export type RemixClient = PluginApi<Readonly<IRemixApi>> & PluginClient<Api, Readonly<IRemixApi>>

export const RemixContext = React.createContext({
  client: {} as unknown as RemixClient,
  isLoaded: false
})

export function RemixProvider ({ children }: { children: ReactNode }): JSX.Element {
  const [pluginLoaded, setPluginLoaded] = useState(false)

  const [client, setClient] = useState(undefined as unknown as RemixClient)

  const clientRef = useRef(client)
  clientRef.current = client

  useEffect(() => {
    console.log('Remix zkSync Plugin loading...')
    const client = createClient(new PluginClient())
    const loadClient = async (): Promise<void> => {
      await client.onload()
      setClient(client)
      console.log('Remix zkSync Plugin has been loaded')
      setPluginLoaded(true)
    }

    void loadClient()
  }, [])

  return (
    <RemixContext.Provider value={{ client, isLoaded: pluginLoaded }}>
      { children }
    </RemixContext.Provider>
  )
}
