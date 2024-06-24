import React from 'react'
import './overlay.css'

export const FullScreenOverlay = ({ children }: { children: React.ReactNode }) => {
  return <div className={'full-overlay'}>{children}</div>
}

export default FullScreenOverlay
