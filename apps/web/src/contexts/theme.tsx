import { createContext, useCallback, useMemo } from 'react'

type State = {
  update(identifier: string): void
}

const ThemeContext = createContext<State>({} as State)

export function ThemeProvider({ children }: React.PropsWithChildren) {
  const update = useCallback((identifier: string) => {
    console.log(identifier)
  }, [])

  const state = useMemo<State>(() => {
    return {
      update
    }
  }, [update])

  return <ThemeContext.Provider value={state}>{children}</ThemeContext.Provider>
}
