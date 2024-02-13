import { useEffect } from 'react'

import { Outlet } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'

import { ThemeProvider } from '#/contexts/theme'

import { HeaderStrip } from '#/components/header-strip'

const client = new QueryClient()
export function MainLayout() {
  useEffect(() => {
    localStorage.setItem('TanstackQueryDevtools.theme_preference', 'dark')
  }, [])

  return (
    <QueryClientProvider client={client}>
      <ThemeProvider>
        <HeaderStrip />
        <Outlet />
      </ThemeProvider>
      <ReactQueryDevtools />
    </QueryClientProvider>
  )
}
