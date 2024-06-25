import React from 'react'
import './container.css'

interface IContainer {
  children?: React.ReactNode
}

export const Container = ({ children }: IContainer) => {
  return <div className="p-2">{children}</div>
}

export default Container
