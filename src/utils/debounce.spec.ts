import { beforeEach, describe, expect, it, vi } from 'vitest'

import { debounce } from './debounce'

describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  it('delays callback execution until the configured time elapses', () => {
    const callback = vi.fn<(value: string) => void>()
    const debounced = debounce(callback, 200)

    debounced('dummy')

    expect(callback).not.toHaveBeenCalled()

    vi.advanceTimersByTime(199)

    expect(callback).not.toHaveBeenCalled()

    vi.advanceTimersByTime(1)

    expect(callback).toHaveBeenCalledWith('dummy')
    expect(callback).toHaveBeenCalledTimes(1)
  })

  it('only invokes the latest call while a debounce timer is pending', () => {
    const callback = vi.fn<(value: string) => void>()
    const debounced = debounce(callback, 200)

    debounced('first')
    vi.advanceTimersByTime(100)

    debounced('second')
    vi.advanceTimersByTime(100)

    expect(callback).not.toHaveBeenCalled()

    vi.advanceTimersByTime(100)

    expect(callback).toHaveBeenCalledTimes(1)
    expect(callback).toHaveBeenCalledWith('second')
  })

  it('can schedule a new call after the previous timer has completed', () => {
    const callback = vi.fn<(value: string) => void>()
    const debounced = debounce(callback, 100)

    debounced('first')
    vi.runOnlyPendingTimers()

    debounced('second')
    vi.runOnlyPendingTimers()

    expect(callback).toHaveBeenNthCalledWith(1, 'first')
    expect(callback).toHaveBeenNthCalledWith(2, 'second')
    expect(callback).toHaveBeenCalledTimes(2)
  })
})
