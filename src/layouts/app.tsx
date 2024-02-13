import { Outlet } from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'

import { client } from '#/lib/react-query'

import { ThemeProvider } from '#/contexts/theme'

import { HeaderStrip } from '#/components/header-strip'

export function AppLayout() {
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
