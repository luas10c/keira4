import { createHashRouter, RouterProvider } from 'react-router-dom'

import { MainLayout } from './layouts/main'

import { ReactComponent as LogoMinimal } from '#/assets/logo-minimal.svg'

import { ReactComponent as ChromeClose } from '#/assets/chrome-close.svg'
import { ReactComponent as ChromeMinimize } from '#/assets/chrome-minimize.svg'

import { ReactComponent as Check } from '#/assets/check.svg'

const router = createHashRouter([
  {
    path: '/',
    element: <MainLayout />,
    children: [
      {
        path: '/',
        element: (
          <div className="flex flex-1 flex-col overflow-hidden bg-bunker-950 text-bunker-100">
            <header className="flex items-center px-2.5 py-2 text-cyan-500">
              <div className="flex flex-1 items-center gap-2.5">
                <LogoMinimal />
                <nav className="flex h-full items-center">
                  <ul className="flex gap-2.5">
                    <li>
                      <a href="/#">
                        <span className="text-sm text-bunker-500 transition-colors hover:text-zinc-400">
                          File
                        </span>
                      </a>
                    </li>
                    <li>
                      <a href="/#">
                        <span className="text-sm text-bunker-500 transition-colors hover:text-zinc-400">
                          Preferences
                        </span>
                      </a>
                    </li>
                    <li>
                      <a href="/#">
                        <span className="text-sm text-bunker-500 transition-colors hover:text-zinc-400">
                          Help
                        </span>
                      </a>
                    </li>
                  </ul>
                </nav>
              </div>

              <div>
                <ul className="flex items-center gap-2.5">
                  <li>
                    <button
                      type="button"
                      className="flex size-6 items-center justify-center text-bunker-500 transition-colors hover:text-bunker-400">
                      <span className="sr-only">Minimize</span>
                      <ChromeMinimize />
                    </button>
                  </li>
                  <li>
                    <button
                      type="button"
                      className="flex size-6 items-center justify-center text-bunker-500 transition-colors hover:text-bunker-400">
                      <span className="sr-only">Close</span>
                      <ChromeClose />
                    </button>
                  </li>
                </ul>
              </div>
            </header>

            <div className="flex flex-1 flex-col items-center justify-center">
              <h2 className="text-white">Welcome!</h2>
              <span className="text-sm text-bunker-400">
                Contribute to our alpha version
              </span>

              <div className="py-4">
                <button
                  type="button"
                  className="rounded bg-cyan-400 px-5 py-2 transition-colors hover:bg-cyan-400/80">
                  <span className="text-bunker-950">Try it now</span>
                </button>
              </div>

              <div className="mt-16 flex space-x-4">
                <ul className="flex space-x-4">
                  <li>
                    <button
                      type="button"
                      className="relative w-[150px] shrink-0 rounded border-2 border-bunker-900 px-4 py-2.5 text-left">
                      <h3 className="text-sm">Light</h3>
                      <span className="text-xs text-bunker-400">....</span>
                    </button>
                  </li>
                  <li>
                    <button
                      type="button"
                      className="relative w-[150px] shrink-0 rounded border-2 border-bunker-900 px-4 py-2.5 text-left">
                      <h3 className="text-sm">Dark</h3>
                      <span className="text-xs text-bunker-400">....</span>
                    </button>
                  </li>
                  <li>
                    <button
                      type="button"
                      className="relative w-[150px] shrink-0 rounded border-2 border-cyan-400 px-4 py-2.5 text-left">
                      <h3 className="text-sm">System</h3>
                      <span className="text-xs text-bunker-400">....</span>

                      <Check className="absolute right-4 top-3 size-4 text-cyan-400" />
                    </button>
                  </li>
                </ul>
              </div>

              <div className="my-8">
                <div className="w-[412px] rounded ring-2 ring-bunker-900 transition-colors focus-within:ring-cyan-400">
                  <input
                    type="text"
                    className="w-full border-none bg-transparent outline-none focus:ring-0"
                  />
                </div>
              </div>

              <div className="w-[348px] rounded bg-bunker-900/50 px-4">
                <header className="py-4 text-center">
                  <h3 className="mx-auto max-w-44">
                    A new version of Keira is available!
                  </h3>
                </header>

                <div className="text-center">
                  <span className="text-sm">
                    Whould you like to update your app now?
                  </span>

                  <div className="flex space-x-4 py-4">
                    <button
                      type="button"
                      className="flex-1 rounded bg-bunker-900">
                      <span className="text-sm">Later</span>
                    </button>

                    <button
                      type="button"
                      className="flex-1 rounded bg-bunker-900">
                      <span className="text-sm">Restart</span>
                    </button>
                  </div>
                </div>
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
