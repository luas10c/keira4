import { createBrowserRouter, RouterProvider } from 'react-router-dom'

import { ReactComponent as LogoMinimal } from '#/assets/logo-minimal.svg'

import { ReactComponent as ChromeMinimize } from '#/assets/chrome-minimize.svg'
import { ReactComponent as ChromeClose } from '#/assets/chrome-close.svg'

import { AppLayout } from './layouts/app'

const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      {
        path: '/',
        element: (
          <div className="relative flex min-h-screen flex-1 flex-col text-bunker-100">
            <header className="flex items-center px-2.5 py-2">
              <div
                className="flex flex-1 items-center gap-2.5"
                data-tauri-drag-region>
                <LogoMinimal className="text-pelorous-500" />

                <nav className="flex">
                  <ul className="flex items-center">
                    <li>
                      <button
                        type="button"
                        className="flex cursor-pointer select-none items-center justify-center rounded px-2.5 py-1 outline-none transition-colors focus-within:bg-bunker-800/60 hover:bg-bunker-800">
                        <span className="text-sm text-bunker-400">File</span>
                      </button>
                    </li>
                    <li>
                      <button
                        type="button"
                        className="flex cursor-pointer select-none items-center justify-center rounded px-2.5 py-1 outline-none transition-colors focus-within:bg-bunker-800/60 hover:bg-bunker-800">
                        <span className="text-sm text-bunker-400">Window</span>
                      </button>
                    </li>
                    <li>
                      <button
                        type="button"
                        className="flex cursor-pointer select-none items-center justify-center rounded px-2.5 py-1 outline-none transition-colors focus-within:bg-bunker-800/60 hover:bg-bunker-800">
                        <span className="text-sm text-bunker-400">Help</span>
                      </button>
                    </li>
                  </ul>
                </nav>
              </div>

              <div>
                <ul className="flex items-center gap-2">
                  <li>
                    <button
                      type="button"
                      data-target="minimize"
                      className="flex size-8 items-center justify-center text-bunker-500 transition-colors hover:text-bunker-400">
                      <span className="sr-only">Minimize</span>
                      <ChromeMinimize className="size-4" />
                    </button>
                  </li>
                  <li>
                    <button
                      type="button"
                      data-target="close"
                      className="flex size-8 items-center justify-center rounded text-bunker-500 transition-colors hover:bg-bunker-900 hover:text-bunker-400">
                      <span className="sr-only">Close</span>
                      <ChromeClose className="size-4" />
                    </button>
                  </li>
                </ul>
              </div>
            </header>

            <div className="relative flex flex-1 flex-col items-center justify-center">
              <h2 className="text-lg text-bunker-100">Welcome!</h2>
              <span className="text-sm text-bunker-400">
                Contribute to our alpha version
              </span>

              <div className="w-full max-w-lg space-y-4 py-4">
                <input
                  type="text"
                  className="flex w-full items-center rounded border-none bg-bunker-900 py-2 leading-relaxed text-bunker-400 ring-2 ring-bunker-800 transition-colors placeholder:text-bunker-600 focus-within:ring-2 focus-within:ring-cyan-500"
                  placeholder="Ex: world"
                />
              </div>

              <div className="py-4">
                <button
                  type="button"
                  className="flex gap-2 rounded bg-pelorous-400 px-5 py-2 transition-colors hover:bg-pelorous-400/80 focus:outline-none focus:ring-2 focus:ring-cyan-500 focus:ring-offset-2 focus:ring-offset-bunker-800 disabled:cursor-not-allowed disabled:bg-pelorous-400/60">
                  {/*             <div className="size-6 animate-spin rounded-full border-2 border-bunker-900 border-l-transparent"></div> */}
                  <span className="text-bunker-950">Try it now</span>
                </button>
              </div>
            </div>
          </div>
        )
      }
    ]
  }
])

export function App() {
  return <RouterProvider router={router} />
}
