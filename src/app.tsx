import { createHashRouter, RouterProvider } from 'react-router-dom'

import { AppLayout } from './layouts/app'
import { MainLayout } from './layouts/main'

import { Connection } from './screens/connection'

const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      {
        path: '/',
        element: <Connection />
      },
      {
        path: '/dashboards',
        element: <MainLayout />,
        children: []
      }
    ]
  }
])

export function App() {
  return <RouterProvider router={router} />
}
