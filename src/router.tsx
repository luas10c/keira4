import {
  createRootRoute,
  createRoute,
  createRouter
} from '@tanstack/react-router'

import { AppLayout } from './layouts/app'
import { MainLayout } from './layouts/main'

import { Home } from './screens/home'

const rootRoute = createRootRoute({
  component: AppLayout
})

const mainLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'main-layout',
  component: MainLayout
})

const homeRoute = createRoute({
  getParentRoute: () => mainLayoutRoute,
  path: '/',
  component: Home
})

const routeTree = rootRoute.addChildren([
  mainLayoutRoute.addChildren([homeRoute])
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
