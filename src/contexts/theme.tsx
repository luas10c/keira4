'use client'

import { createContext, useEffect, useMemo } from 'react'

const ThemeContext = createContext({})

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    //
  }, [])

  const state = useMemo(() => {
    return {}
  }, [])

  return <ThemeContext.Provider value={state}>{children}</ThemeContext.Provider>
}
