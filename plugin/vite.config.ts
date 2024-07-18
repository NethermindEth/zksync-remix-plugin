import { defineConfig } from 'vite'
import path from 'path'
import react from '@vitejs/plugin-react'
import svgr from 'vite-plugin-svgr'
import checker from 'vite-plugin-checker'
import cp from 'child_process'

import packageJson from './package.json'

const commitHash = cp.execSync('git rev-parse --short HEAD').toString().replace('\n', '')

const APP_VERSION = `${packageJson.version}-${commitHash}`

export default defineConfig(() => {
  return {
    define: {
      'process.env': {},
      'import.meta.env.APP_VERSION': JSON.stringify(APP_VERSION)
    },
    server: {
      port: 3000,
      hmr: true
    },
    build: {
      outDir: 'build'
    },
    plugins: [
      react(),
      svgr({ svgrOptions: { icon: true } }),
      checker({
        typescript: true,
        eslint: {
          lintCommand: 'eslint "./src/**/*.{ts,tsx}"' // for example, lint .ts & .tsx
        }
      })
    ],
    resolve: {
      alias: {
        '@/': `${path.resolve(__dirname, 'src')}/`
      }
    }
  }
})
