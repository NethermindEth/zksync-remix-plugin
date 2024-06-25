import React from 'react'

import './index.css'
import { MdCheckCircleOutline, MdErrorOutline } from 'react-icons/md'
import { Tooltip } from '@/ui_components'

interface IStateAction {
  value?: 'loading' | 'success' | 'error' | ''
  errorTooltipText?: string
}

const StateAction: React.FC<IStateAction> = ({ value, errorTooltipText }) => {
  switch (value) {
    case 'loading':
      return <span className="spinner-border spinner-border-sm" role="status" aria-hidden="true" />
    case 'success':
      return <MdCheckCircleOutline color="green" size={18} />
    case 'error':
      return (
        <Tooltip
          icon={<MdErrorOutline color="red" size={18} />}
          content={<p>{errorTooltipText}</p>}
          side="right"
          sideOffset={-4}
          contentClassName="max-w-300"
        />
      )
    default:
      return <></>
  }
}

export default StateAction
