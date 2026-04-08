import { vi } from 'vitest'

const tauriMocks = {
  // @tauri-apps/api/core
  invoke: vi.fn(),

  // @tauri-apps/plugin-os
  platform: vi.fn(),

  // @tauri-apps/api/window
  currentWindow: {
    isAlwaysOnTop: vi.fn(),
    isMaximized: vi.fn(),
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    isClosable: vi.fn(),
    close: vi.fn(),
    isFullscreen: vi.fn(),
    isMaximizable: vi.fn(),
    isMinimizable: vi.fn(),
    isMinimized: vi.fn(),
    isResizable: vi.fn(),
    emit: vi.fn(),
    emitTo: vi.fn(),
    listen: vi.fn(),
    once: vi.fn(),
    unlisten: vi.fn(),
    onCloseRequested: vi.fn(),
    onDragDropEvent: vi.fn(),
    onFocusChanged: vi.fn(),
    onResized: vi.fn(),
    onMoved: vi.fn(),
    startDragging: vi.fn(),
    startResizeDragging: vi.fn(),
    requestUserAttention: vi.fn(),
    setFullscreen: vi.fn(),
    setPosition: vi.fn()
  },

  // @tauri-apps/plugin-opener
  openUrl: vi.fn(),
  openPath: vi.fn(),
  revealItemInDir: vi.fn(),

  // @tauri-apps/plugin-positioner
  moveWindow: vi.fn(),
  Position: {
    TopLeft: 'TopLeft',
    TopRight: 'TopRight',
    BottomLeft: 'BottomLeft',
    BottomRight: 'BottomRight',
    Center: 'Center'
  },

  // @tauri-apps/plugin-process
  exit: vi.fn(),
  relaunch: vi.fn(),

  // @tauri-apps/plugin-updater
  check: vi.fn()
}

vi.mock('@tauri-apps/api/core', () => ({
  invoke: tauriMocks.invoke
}))

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: tauriMocks.platform
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => tauriMocks.currentWindow,
  Window: {
    getCurrent: () => tauriMocks.currentWindow,
    getAll: vi.fn().mockResolvedValue([tauriMocks.currentWindow]),
    getByLabel: vi.fn().mockResolvedValue(tauriMocks.currentWindow),
    getFocusedWindow: vi.fn().mockResolvedValue(tauriMocks.currentWindow)
  }
}))

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: tauriMocks.openUrl,
  openPath: tauriMocks.openPath,
  revealItemInDir: tauriMocks.revealItemInDir
}))

vi.mock('@tauri-apps/plugin-positioner', () => ({
  moveWindow: tauriMocks.moveWindow,
  Position: tauriMocks.Position
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  exit: tauriMocks.exit,
  relaunch: tauriMocks.relaunch
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: tauriMocks.check
}))

export function getTauriMocks() {
  return tauriMocks
}

export function resetTauriMocks() {
  tauriMocks.invoke.mockReset()
  tauriMocks.invoke.mockResolvedValue(undefined)

  tauriMocks.platform.mockReturnValue('linux')

  // --- //

  const unlisten = vi.fn()

  for (const value of Object.values(tauriMocks.currentWindow)) {
    if (vi.isMockFunction(value)) {
      value.mockReset()
      value.mockResolvedValue(undefined)
    }
  }

  tauriMocks.currentWindow.isAlwaysOnTop.mockResolvedValue(false)
  tauriMocks.currentWindow.isMaximized.mockResolvedValue(false)
  tauriMocks.currentWindow.isClosable.mockResolvedValue(true)
  tauriMocks.currentWindow.isFullscreen.mockResolvedValue(false)
  tauriMocks.currentWindow.isMaximizable.mockResolvedValue(true)
  tauriMocks.currentWindow.isMaximized.mockResolvedValue(false)
  tauriMocks.currentWindow.isMinimizable.mockResolvedValue(true)
  tauriMocks.currentWindow.isMinimized.mockResolvedValue(false)
  tauriMocks.currentWindow.isResizable.mockResolvedValue(true)
  tauriMocks.currentWindow.listen.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.once.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.listen.mockResolvedValue(
    tauriMocks.currentWindow.unlisten
  )
  tauriMocks.currentWindow.onCloseRequested.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.onDragDropEvent.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.onFocusChanged.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.onResized.mockResolvedValue(unlisten)
  tauriMocks.currentWindow.onMoved.mockResolvedValue(unlisten)

  // ---- //

  tauriMocks.openUrl.mockReset()
  tauriMocks.openPath.mockReset()
  tauriMocks.revealItemInDir.mockReset()

  tauriMocks.openUrl.mockResolvedValue(undefined)
  tauriMocks.openPath.mockResolvedValue(undefined)
  tauriMocks.revealItemInDir.mockResolvedValue(undefined)

  tauriMocks.moveWindow.mockReset()
  tauriMocks.moveWindow.mockResolvedValue(undefined)

  tauriMocks.exit.mockReset()
  tauriMocks.relaunch.mockReset()

  tauriMocks.exit.mockResolvedValue(undefined)
  tauriMocks.relaunch.mockResolvedValue(undefined)

  tauriMocks.check.mockReset()
  tauriMocks.check.mockResolvedValue(null)
}
