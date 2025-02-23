import { createBrowserRouter, type RouteObject } from 'react-router'

import { AppLayout } from './layouts/app'
import { MainLayout } from './layouts/main'

import { Home } from './screens/home'
import { Connection } from './screens/connection'
import { Preferences } from './screens/preferences'

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
          },
          {
            path: '/connection',
            element: <Connection />
          },
          {
            path: '/preferences',
            element: <Preferences />
          }
        ]
      }
    ]
  }
]

export const router = createBrowserRouter(routes)
