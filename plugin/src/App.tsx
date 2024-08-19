import React from 'react'
import { Provider } from 'jotai'
import './App.css'
import { Plugin } from '@/features/Plugin'
import { remixClientStore } from '@/stores/remixClient'

const App = () => {
  return (
    <div className="shell bg-primary">
      <Provider store={remixClientStore}>
        <Plugin />
      </Provider>
    </div>
  )
}

export default App
