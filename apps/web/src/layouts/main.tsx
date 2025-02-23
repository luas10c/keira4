import { useHotkeys } from '@tanstack/react-hotkeys'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Outlet } from '@tanstack/react-router'
import { exit, relaunch } from '@tauri-apps/plugin-process'
import { getCurrentWindow } from '@tauri-apps/api/window'

const client = new QueryClient()

export function MainLayout() {
  useHotkeys([
    {
      hotkey: 'Control+R',
      async callback() {
        await relaunch()
      }
    },
    {
      hotkey: 'Control+Q',
      async callback() {
        await exit()
      }
    },
    {
      hotkey: 'F11',
      async callback() {
        const currentWindow = getCurrentWindow()
        const isFullscreen = await currentWindow.isFullscreen()
        await currentWindow.setFullscreen(!isFullscreen)
      }
    }
  ])

  return (
    <QueryClientProvider client={client}>
      <div className="flex h-full flex-1">
        {/* .sidebar */}
        <main className="flex flex-1 px-4">
          <Outlet />
        </main>
      </div>
    </QueryClientProvider>
  )
}
