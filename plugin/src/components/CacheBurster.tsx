import React, { createContext, useCallback, useContext, useEffect, useState } from 'react'
import { compare } from 'compare-versions'

const CacheBusterContext = createContext<() => Promise<void>>(async () => undefined)

type CacheBusterPropsType = {
  children: React.ReactNode
  currentVersion: string
  isEnabled: boolean
  isVerboseMode: boolean
  loadingComponent: React.ReactNode
  metaFileDirectory: string | null
}

export const CacheBuster = ({
  children = null,
  currentVersion,
  isEnabled = false,
  isVerboseMode = false,
  loadingComponent = null,
  metaFileDirectory = null
}: CacheBusterPropsType) => {
  const [cacheStatus, setCacheStatus] = useState({
    loading: true,
    isLatestVersion: false
  })

  const log = useCallback(
    (message: string, isError?: boolean) => {
      isVerboseMode && (isError ? console.error(message) : console.log(message))
    },
    [isVerboseMode]
  )

  const getMetaFileDirectory = useCallback(() => {
    return !metaFileDirectory || metaFileDirectory === '.' ? '' : metaFileDirectory
  }, [metaFileDirectory])

  const checkCacheStatus = useCallback(async () => {
    try {
      const res = await fetch(`${getMetaFileDirectory()}/meta.json`)
      const { version: metaVersion } = await res.json()

      const shouldForceRefresh = isThereNewVersion(metaVersion, currentVersion)
      if (shouldForceRefresh) {
        log(`There is a new version (v${metaVersion}). Should force refresh.`)
        setCacheStatus({
          loading: false,
          isLatestVersion: false
        })
      } else {
        log('There is no new version. No cache refresh needed.')
        setCacheStatus({
          loading: false,
          isLatestVersion: true
        })
      }
    } catch (error) {
      log('An error occurred while checking cache status.', true)
      log(error as string, true)

      !isVerboseMode &&
        setCacheStatus({
          loading: false,
          isLatestVersion: true
        })
    }
  }, [currentVersion, getMetaFileDirectory, isVerboseMode, log])

  useEffect(() => {
    isEnabled ? checkCacheStatus() : log('React Cache Buster is disabled.')
  }, [checkCacheStatus, isEnabled, log])

  const isThereNewVersion = (metaVersion: string, currentVersion: string) => {
    return compare(metaVersion, currentVersion, '>')
  }

  const refreshCacheAndReload = async () => {
    try {
      if (window?.caches) {
        const { caches } = window
        const cacheNames = await caches.keys()
        const cacheDeletionPromises = cacheNames.map((n) => caches.delete(n))

        await Promise.all(cacheDeletionPromises)
        log('The cache has been deleted.')
        window.location.reload()
      }
    } catch (error) {
      log('An error occurred while deleting the cache.', true)
      log(error as string, true)
    }
  }

  if (!isEnabled) {
    return children
  } else {
    if (cacheStatus.loading) {
      return loadingComponent
    }

    if (!cacheStatus.loading && !cacheStatus.isLatestVersion) {
      refreshCacheAndReload()
      return null
    }

    return <CacheBusterContext.Provider value={checkCacheStatus}>{children}</CacheBusterContext.Provider>
  }
}

export const useCacheBuster = () => {
  const context = useContext(CacheBusterContext)
  if (context === undefined || context === null) {
    throw new Error('useCacheBuster must be used within a CacheBuster component.')
  }
  return context
}
