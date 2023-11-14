import React, { useEffect, useState } from 'react'
import './App.css'
import Plugin from './features/Plugin'
import Loader from './ui_components/CircularLoader'
import FullScreenOverlay from './ui_components/FullScreenOverlay'
import { useAtomValue } from 'jotai/react/useAtomValue'
import { pluginLoaded } from './atoms/remixClient'

const App: React.FC = () => {
  return (
    <div className="shell">
      {useAtomValue(pluginLoaded)
        ? (
          <Plugin />
          )
        : (
        <FullScreenOverlay>
          <Loader />
        </FullScreenOverlay>
          )}
    </div>
  )
}

export default App
