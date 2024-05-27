import { useEffect, useRef } from 'react'

type CallbackFunction = () => void

const useInterval = (callback: CallbackFunction, delay: number | null) => {
  const savedCallback = useRef<CallbackFunction>(() => null)

  useEffect(() => {
    savedCallback.current = callback
  })

  useEffect(() => {
    if (delay !== null) {
      const interval = setInterval(() => {
        savedCallback.current()
      }, delay)
      return () => {
        clearInterval(interval)
      }
    }

    return undefined
  }, [delay])
}

export default useInterval
