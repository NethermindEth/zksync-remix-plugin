import React from 'react'
import './container.css'
import clsx from 'clsx'

interface IContainer {
  children?: React.ReactNode
  className?: string
}

export const Container = ({ children, className }: IContainer) => {
  return <div className={clsx('p-2', className)}>{children}</div>
}

export default Container
