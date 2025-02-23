import { createBrowserRouter, type RouteObject } from 'react-router'

import { AppLayout } from './layouts/app'
import { MainLayout } from './layouts/main'

import { Home } from './screens/home'

const routes: RouteObject[] = [
  {
    element: <AppLayout />,
    children: [
      {
        element: <MainLayout />,
        children: [
          {
            path: '/',
            element: <Home />
          }
        ]
      }
    ]
  }
]

export const router = createBrowserRouter(routes)
