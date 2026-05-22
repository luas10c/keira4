import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { createRef } from 'react'

import { Scrollable } from '.'

function NonRefChild() {
  return <section>No forwarded ref</section>
}

describe('Scrollable', () => {
  it('renders a default div root with the expected classes', () => {
    render(
      <Scrollable className="h-40">
        <div>Scrollable content</div>
      </Scrollable>
    )

    const root = screen.getByRole('region', { name: 'Scrollable' })

    expect(root.tagName).toBe('DIV')
    expect(root).toHaveClass('relative', 'overflow-hidden', 'h-40')
    expect(root).toHaveAttribute('data-overlayscrollbars', 'host')
  })

  it('forwards refs to the default root element', () => {
    const ref = createRef<HTMLDivElement>()

    render(
      <Scrollable ref={ref}>
        <div>Scrollable content</div>
      </Scrollable>
    )

    expect(ref.current).toBe(
      screen.getByRole('region', { name: 'Scrollable' })
    )
  })

  it('cleans up the overlayscrollbars DOM on unmount', () => {
    const { unmount } = render(
      <Scrollable>
        <div>Scrollable content</div>
      </Scrollable>
    )

    const root = screen.getByRole('region', { name: 'Scrollable' })

    expect(root).toHaveAttribute('data-overlayscrollbars', 'host')

    unmount()

    expect(screen.queryByRole('region', { name: 'Scrollable' })).toBeNull()
  })

  it('skips initialization when the slotted child cannot receive a ref', () => {
    render(
      <Scrollable asChild>
        <NonRefChild />
      </Scrollable>
    )

    const section = screen.getByText('No forwarded ref')

    expect(section).toHaveTextContent('No forwarded ref')
    expect(section).not.toHaveAttribute('data-overlayscrollbars', 'host')
  })
})
