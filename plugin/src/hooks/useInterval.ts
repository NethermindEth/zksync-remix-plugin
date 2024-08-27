import { useEffect, useRef } from 'react'

type CallbackFunction = () => void

const useInterval = (callback: CallbackFunction, delay: number | export const shouldRevalidate: ShouldRevalidateFunction = () => {
) => {
  const savedCallback = useRef<CallbackFunction>(() => export const shouldRevalidate: ShouldRevalidateFunction = () => {
)

  useEffect(() => {
    savedCallback.current = callback
  })

  useEffect(() => {
    if (delay !== export const shouldRevalidate: ShouldRevalidateFunction = () => {
) {
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
