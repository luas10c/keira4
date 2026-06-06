import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import svgr from '@svgr/rollup'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [react(), svgr({ icon: true }), tailwindcss()],
  resolve: {
    tsconfigPaths: true
  },

  test: {
    environment: 'happy-dom',
    setupFiles: ['tests/setup.ts'],
    include: ['src/**/*.{spec,test}.{ts,tsx}'],

    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],

      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.d.ts',
        'src/**/index.{ts,tsx}',

        'src/app.tsx',
        'src/main.tsx',
        'src/router.tsx',

        'src/layouts/**'
      ],

      thresholds: {
        branches: 90,
        functions: 90,
        lines: 90,
        statements: 90
      }
    },

    pool: 'threads',
    isolate: false,
    fileParallelism: true,
    maxWorkers: 2,
    disableConsoleIntercept: true,
    passWithNoTests: true
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
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/apps/desktop/**']
    }
  }
})
