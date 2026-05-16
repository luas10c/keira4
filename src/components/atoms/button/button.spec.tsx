import { vi, describe, it, expect } from 'vitest'
import { screen, render } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { Button } from '.'

describe('Button Tests', () => {
  it('should renders correctly', () => {
    render(
      <Button type="button">
        <span>Aaa</span>
      </Button>
    )

    expect(screen.getByRole('button', { name: /Aaa/i })).toBeInTheDocument()
  })

  it('should be able clicks', async () => {
    const handle = vi.fn()

    render(
      <Button type="button" onClick={handle}>
        <span>Aaa</span>
      </Button>
    )

    await userEvent.click(screen.getByRole('button', { name: /Aaa/i }))

    expect(handle).toHaveBeenCalledTimes(1)
  })

  it('should not be clickable when disabled', async () => {
    const handle = vi.fn()

    render(
      <Button type="button" onClick={handle} disabled>
        <span>Aaa</span>
      </Button>
    )

    await userEvent.click(screen.getByRole('button', { name: /Aaa/i }))

    expect(handle).toHaveBeenCalledTimes(0)
  })

  it('should render child element instead of button when asChild is true', () => {
    render(
      <Button asChild>
        <a href="/home">Home</a>
      </Button>
    )

    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Home/i })).toBeInTheDocument()
  })

  it('should forward button className to child element', () => {
    render(
      <Button asChild variant="primary" size="lg">
        <a href="/home">Home</a>
      </Button>
    )

    const link = screen.getByRole('link', { name: /Home/i })

    expect(link.className).toMatch(/bg-cyan-500/)
    expect(link.className).toMatch(/h-11/)
  })

  it('should merge child className with button className', () => {
    render(
      <Button asChild className="extra-class">
        <a href="/home" className="child-class">
          Home
        </a>
      </Button>
    )

    const link = screen.getByRole('link', { name: /Home/i })

    expect(link.className).toContain('extra-class')
    expect(link.className).toContain('child-class')
  })

  it('should compose onClick handlers from both button and child', async () => {
    const buttonHandle = vi.fn()
    const childHandle = vi.fn()

    render(
      <Button asChild onClick={buttonHandle}>
        <a href="/home" onClick={childHandle}>
          Home
        </a>
      </Button>
    )

    await userEvent.click(screen.getByRole('link', { name: /Home/i }))

    expect(buttonHandle).toHaveBeenCalledTimes(1)
    expect(childHandle).toHaveBeenCalledTimes(1)
  })

  it('should forward ref to child element when asChild is true', () => {
    const ref = vi.fn()

    render(
      <Button asChild ref={ref}>
        <a href="/home">Home</a>
      </Button>
    )

    expect(ref).toHaveBeenCalledWith(expect.any(HTMLAnchorElement))
  })

  it('should render normally when asChild is false', () => {
    render(<Button>Label</Button>)

    expect(screen.getByRole('button', { name: /label/i })).toBeInTheDocument()
  })
})
