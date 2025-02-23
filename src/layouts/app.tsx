import { QueryClientProvider, QueryClient } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet } from 'react-router'

import { I18nextProvider } from 'react-i18next'
import i18n from 'i18next'

const client = new QueryClient()

export function AppLayout() {
  return (
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={client}>
        <Outlet />

        <ReactQueryDevtools position="right" />
      </QueryClientProvider>
    </I18nextProvider>
  )
}
