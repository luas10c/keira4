export function getDeviceThemeFromMedia(): 'dark' | 'light' {
  const { matches } = window.matchMedia('(prefers-color-scheme: dark)')
  if (matches) return 'dark'

  return 'light'
}
