export function debounce<T>(cb: (...args: T[]) => void, delay: number) {
  let timer: ReturnType<typeof setTimeout> | null

  return (...args: T[]) => {
    if (timer) {
      clearTimeout(timer)
    }

    timer = setTimeout(() => {
      cb(...args)
      timer = null
    }, delay)
  }
}
