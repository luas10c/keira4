import { createContext, useMemo } from 'react'

type AuthStatus = 'authenticated' | 'success' | 'failed'

type State = {
  status: AuthStatus
}

const AuthContext = createContext<State>({} as State)

export function AuthProvider({ children }: React.PropsWithChildren) {
  const state = useMemo<State>(() => {
    return {
      status: 'success'
    }
  }, [])

  return <AuthContext.Provider value={state}>{children}</AuthContext.Provider>
}
