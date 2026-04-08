import { describe, it, expect } from 'vitest'

import { cn } from './cn'

describe('cn', () => {
  it('should merge class names correctly', () => {
    expect(cn('flex', 'items-center', 'justify-center')).toBe(
      'flex items-center justify-center'
    )
  })

  it('should ignore falsy values', () => {
    expect(cn('flex', false, null, undefined, '', 'items-center')).toBe(
      'flex items-center'
    )
  })

  it('should support conditional class names', () => {
    const truphy = true
    const falsy = false
    expect(
      cn('button', truphy && 'button-active', falsy && 'button-disabled')
    ).toBe('button button-active')
  })

  it('should resolve Tailwind class conflicts', () => {
    expect(cn('px-2', 'px-4')).toBe('px-4')
  })
})
