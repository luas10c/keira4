import { createHashRouter, RouterProvider } from 'react-router-dom'

import { MainLayout } from './layouts/main'

import { ReactComponent as LogoMinimal } from '#/assets/logo-minimal.svg'

const router = createHashRouter([
  {
    path: '/',
    element: <MainLayout />,
    children: [
      {
        path: '/',
        element: (
          <div className="flex flex-1 flex-col overflow-hidden rounded-lg border-2 border-cyan-500 bg-bunker-950 text-bunker-100">
            <header className="flex items-center gap-2.5 px-2.5 py-2 text-cyan-500">
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
            </header>

            <div className="flex flex-1 flex-col items-center justify-center">
              <h2>Welcome!</h2>
              <span className="text-sm text-zinc-500">
                Contribute to our alpha version
              </span>

              <div className="py-4">
                <button
                  type="button"
                  className="rounded bg-cyan-400 px-5 py-2 transition-colors hover:bg-cyan-400/80">
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
