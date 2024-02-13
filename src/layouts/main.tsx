import { useEffect } from 'react'

import { Outlet } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'

const client = new QueryClient()
export function MainLayout() {
  useEffect(() => {
    localStorage.setItem('TanstackQueryDevtools.theme_preference', 'dark')
  }, [])

  return (
    <QueryClientProvider client={client}>
      <Outlet />
      <ReactQueryDevtools />
    </QueryClientProvider>
  )
}
