import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import * as Radio from '.'

describe('Radio', () => {
  it('renders an accessible radio with default indicator', () => {
    render(<Radio.Root aria-label="Any time" />)

    const root = screen.getByRole('radio', { name: 'Any time' })
    const indicator = root.querySelector('span')

    expect(root).toHaveAttribute('aria-checked', 'false')
    expect(root).toHaveAttribute('data-state', 'unchecked')
    expect(indicator).toHaveAttribute('data-state', 'unchecked')
  })

  it('supports composition with Radio.Indicator', () => {
    render(
      <Radio.Root aria-label="Any time" checked>
        <Radio.Indicator data-testid="indicator" className="custom-indicator" />
      </Radio.Root>
    )

    const indicator = screen.getByTestId('indicator')
    expect(indicator).toHaveAttribute('data-state', 'checked')
    expect(indicator).toHaveClass('custom-indicator')
  })

  it('selects uncontrolled state on click', async () => {
    render(<Radio.Root aria-label="Any time" />)

    const root = screen.getByRole('radio', { name: 'Any time' })

    await userEvent.click(root)

    expect(root).toHaveAttribute('aria-checked', 'true')
  })

  it('calls onCheckedChange when controlled', async () => {
    const handleChange = vi.fn()

    render(
      <Radio.Root
        aria-label="Any time"
        checked={false}
        onCheckedChange={handleChange}
      />
    )

    await userEvent.click(screen.getByRole('radio', { name: 'Any time' }))

    expect(handleChange).toHaveBeenCalledWith(true)
  })

  it('selects with keyboard', async () => {
    render(<Radio.Root aria-label="Any time" />)

    const root = screen.getByRole('radio', { name: 'Any time' })
    root.focus()

    await userEvent.keyboard('[Space]')

    expect(root).toHaveAttribute('aria-checked', 'true')
  })

  it('does not select when disabled', async () => {
    const handleChange = vi.fn()

    render(
      <Radio.Root
        aria-label="Any time"
        disabled
        onCheckedChange={handleChange}
      />
    )

    const root = screen.getByRole('radio', { name: 'Any time' })

    await userEvent.click(root)

    expect(root).toHaveAttribute('aria-disabled', 'true')
    expect(root).toHaveAttribute('tabindex', '-1')
    expect(root).toHaveAttribute('aria-checked', 'false')
    expect(handleChange).not.toHaveBeenCalled()
  })

  it('exposes aria-invalid when provided', () => {
    render(
      <Radio.Root aria-label="Any time" aria-invalid name="availability" />
    )

    expect(document.querySelector('input[type="radio"]')).toHaveAttribute(
      'aria-invalid',
      'true'
    )
  })

  it('does not uncheck a selected standalone radio', async () => {
    const handleChange = vi.fn()

    render(
      <Radio.Root
        aria-label="Any time"
        defaultChecked
        onCheckedChange={handleChange}
      />
    )

    const root = screen.getByRole('radio', { name: 'Any time' })
    await userEvent.click(root)

    expect(root).toHaveAttribute('aria-checked', 'true')
    expect(handleChange).not.toHaveBeenCalled()
  })

  it('associates label with the radio control', async () => {
    render(
      <Radio.Root>
        <Radio.Label>Any time</Radio.Label>
      </Radio.Root>
    )

    const root = screen.getByRole('radio')
    await userEvent.click(screen.getByText('Any time'))

    expect(root).toHaveAttribute('aria-checked', 'true')
  })

  it('renders a native input for form submission', () => {
    render(
      <Radio.Root
        aria-label="Any time"
        name="availability"
        value="any"
        checked
      />
    )

    const input = document.querySelector('input[name="availability"]')

    expect(input).toHaveAttribute('type', 'radio')
    expect(input).toHaveAttribute('value', 'any')
    expect(input).toBeChecked()
  })

  it('supports grouped radios', async () => {
    const handleValueChange = vi.fn()

    render(
      <Radio.Group
        name="availability"
        value="any"
        onValueChange={handleValueChange}
      >
        <Radio.Root aria-label="Any time" value="any" />
        <Radio.Root aria-label="Working hours" value="work" />
      </Radio.Group>
    )

    expect(
      screen.getByRole('radiogroup', { name: 'availability' })
    ).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: 'Any time' })).toHaveAttribute(
      'aria-checked',
      'true'
    )

    await userEvent.click(screen.getByRole('radio', { name: 'Working hours' }))

    expect(handleValueChange).toHaveBeenCalledWith('work')
  })
})
