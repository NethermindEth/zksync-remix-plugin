import React from 'react'
import * as TooltipPrimitive from '@radix-ui/react-tooltip'
import clsx from 'clsx'
import './tooltip.css'

interface ITooltip {
  icon: React.ReactNode
  content: string | React.ReactNode
  side?: 'top' | 'right' | 'bottom' | 'left'
  sideOffset?: number
  contentClassName?: string
  avoidCollisions?: boolean
}

export const Tooltip = ({
  icon,
  content,
  side = 'top',
  sideOffset = 5,
  contentClassName,
  avoidCollisions = true
}: ITooltip) => {
  return (
    <TooltipPrimitive.Provider delayDuration={100}>
      <TooltipPrimitive.Root>
        <TooltipPrimitive.Trigger asChild>
          <button className="text-truncate">{icon}</button>
        </TooltipPrimitive.Trigger>
        <TooltipPrimitive.Portal>
          <TooltipPrimitive.Content
            className={clsx(contentClassName, 'TooltipContent')}
            sideOffset={sideOffset}
            side={side}
            avoidCollisions={avoidCollisions}
          >
            {content}
            <TooltipPrimitive.Arrow className="TooltipArrow" />
          </TooltipPrimitive.Content>
        </TooltipPrimitive.Portal>
      </TooltipPrimitive.Root>
    </TooltipPrimitive.Provider>
  )
}

export default Tooltip
