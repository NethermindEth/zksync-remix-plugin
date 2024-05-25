import React from 'react'
import './App.css'
import Plugin from './features/Plugin'
import { remixClientStore } from './stores/remixClient'
import { Provider } from 'jotai'

const App: React.FC = () => {
  return (
    <div className="shell bg-primary">
      {
        <Provider store={remixClientStore}>
          <Plugin />
        </Provider>
      }
    </div>
  )
}

export default App
