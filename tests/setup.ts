import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach, beforeEach, vi } from 'vitest'

import { resetTauriMocks } from './mocks/tauri'

beforeEach(() => {
  vi.resetModules()
  vi.clearAllMocks()

  resetTauriMocks()
})

afterEach(() => {
  cleanup()
})
