import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react-swc'
import tailwindcss from '@tailwindcss/vite'

import svgr from '@svgr/rollup'
import path from 'node:path'

/** @type {import('vite').UserConfig} */
const config = ({ mode }) => {
  process.env = {
    ...process.env,
    ...loadEnv(mode, process.cwd())
  }

  return defineConfig({
    plugins: [react(), tailwindcss(), svgr()],
    resolve: {
      alias: {
        '#': path.join(import.meta.dirname, 'src')
      }
    },

    envPrefix: [
      'VITE_',
      'TAURI_PLATFORM',
      'TAURI_ARCH',
      'TAURI_FAMILY',
      'TAURI_PLATFORM_VERSION',
      'TAURI_PLATFORM_TYPE',
      'TAURI_DEBUG'
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 3000,
      strictPort: true,
      watch: {
        // 3. tell vite to ignore watching `src-tauri`
        ignored: ['**/src-tauri/**']
      }
    },

    build: {
      // Tauri supports es2023
      target: ['es2023', 'chrome126', 'safari17'],
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG
    }
  })
}
export default config
