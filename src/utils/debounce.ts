export function debounce<T>(callback: (...args: T[]) => void, delay: number) {
  let timer: ReturnType<typeof setTimeout> | null

  return (...args: T[]) => {
    if (timer) {
      clearTimeout(timer)
    }

    timer = setTimeout(() => {
      callback(...args)
      timer = null
    }, delay)
  }
}
