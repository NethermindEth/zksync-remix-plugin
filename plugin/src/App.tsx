import React from 'react'
import { Provider } from 'jotai'
import './App.css'
import { Plugin } from '@/features/Plugin'
import { remixClientStore } from '@/stores/remixClient'
import { CacheBuster } from '@/components/CacheBurster'
import { FullScreenOverlay, Loader } from './ui_components'

const App = () => {
  const isProduction = process.env.NODE_ENV === 'production'
  const APP_VERSION = process.env.APP_VERSION || ''
  console.log({ isProduction, APP_VERSION })
  return (
    <CacheBuster
      currentVersion={APP_VERSION}
      isEnabled={isProduction}
      isVerboseMode={true}
      loadingComponent={
        <FullScreenOverlay>
          <Loader />
        </FullScreenOverlay>
      }
      metaFileDirectory="."
    >
      <div className="shell bg-primary">
        <Provider store={remixClientStore}>
          <Plugin />
        </Provider>
      </div>
    </CacheBuster>
  )
}

export default App
