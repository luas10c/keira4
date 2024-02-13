import { useRef, useEffect } from 'react'
import { appWindow } from '@tauri-apps/api/window'

export function HeaderStrip() {
  const container = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!container.current) return

    container.current.addEventListener('mousedown', appWindow.startDragging)
    container.current.addEventListener('dblclick', appWindow.toggleMaximize)

    return () => {
      if (!container.current) return

      container.current.removeEventListener('mousedown', appWindow.startDragging)
      container.current.removeEventListener('dblclick', appWindow.toggleMaximize)
    }
  }, [])

  return (
    <div className="flex h-9 items-center justify-between border-b-2 border-zinc-700/10 bg-zinc-800/40">
      <div className="h-full">
        {/* <AppIcon /> */}
        <ul className="flex h-full items-center space-x-2 px-4">
          <li>
            <button type="button" className="flex items-center">
              <div className="size-4 rounded-full bg-red-500"></div>
            </button>
          </li>
          <li>
            <button type="button" className="flex items-center">
              <div className="size-4 rounded-full bg-yellow-500"></div>
            </button>
          </li>
          {/* <li>
            <button type="button" className="flex items-center">
              <div className="size-4 rounded-full bg-green-500"></div>
            </button>
          </li> */}
        </ul>
      </div>
      <div ref={container} className="h-full flex-1" />
    </div>
  )
}
