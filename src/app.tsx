import { createHashRouter, RouterProvider } from 'react-router-dom'

import { MainLayout } from './layouts/main'

import { Connection } from './screens/connection'
import { Preferences } from './screens/preferences'

const router = createHashRouter([
  {
    path: '/',
    element: <MainLayout />,
    children: [
      {
        path: '/',
        element: <Connection />
      },
      {
        path: '/preferences',
        element: <Preferences />
      }
    ]
  }
])

export function App() {
  return <RouterProvider router={router} />
}
