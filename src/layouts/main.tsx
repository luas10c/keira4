import { useHotkeys } from '@tanstack/react-hotkeys'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Outlet } from '@tanstack/react-router'
import { exit, relaunch } from '@tauri-apps/plugin-process'
import { getCurrentWindow } from '@tauri-apps/api/window'

const client = new QueryClient()

export function MainLayout() {
  useHotkeys([
    {
      hotkey: 'Mod+R',
      async callback() {
        await relaunch()
      }
    },
    {
      hotkey: 'Mod+Q',
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
      <div className="flex h-dvh flex-1 flex-col">
        <div className="flex flex-1 flex-col overflow-hidden">
          <div className="flex flex-1">
            <main className="flex flex-1">
              <Outlet />
            </main>
          </div>
        </div>
      </div>
    </QueryClientProvider>
  )
}
