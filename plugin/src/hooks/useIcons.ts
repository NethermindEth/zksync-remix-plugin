import { useState, useEffect } from 'react'
import { remixClientAtom } from '@/stores/remixClient'
import { useAtomValue } from 'jotai'

export const useIcon = (name: string): string => {
  const remixClient = useAtomValue(remixClientAtom)
  const [remixTheme, setRemixTheme] = useState('dark')

  useEffect(() => {
    const loadTheme = async (): Promise<void> => {
      try {
        const currentTheme = await remixClient.call('theme', 'currentTheme')
        setRemixTheme(currentTheme.quality ?? 'dark')
      } catch (error) {
        console.error(error)
      }
    }

    const updateTheme = (theme: any): void => {
      setRemixTheme(theme.quality)
    }

    loadTheme().catch(console.error)

    remixClient.on('theme', 'themeChanged', updateTheme)

    return () => {
      remixClient.off('theme', 'themeChanged')
    }
  }, [remixClient])

  return `/${remixTheme}-${name}`
}
