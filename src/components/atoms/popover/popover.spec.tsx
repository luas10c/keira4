import { describe, expect, it, vi } from 'vitest'
import { render, screen, waitForElementToBeRemoved } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import * as Popover from '.'

describe('Popover', () => {
  it('renders closed by default', () => {
    render(
      <Popover.Root>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    expect(screen.getByRole('button', { name: 'Open' })).toHaveAttribute(
      'aria-expanded',
      'false'
    )
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('opens and closes when the trigger is clicked', async () => {
    render(
      <Popover.Root>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    const trigger = screen.getByRole('button', { name: 'Open' })

    await userEvent.click(trigger)

    expect(trigger).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByRole('dialog')).toHaveTextContent('Content')

    await userEvent.click(trigger)

    expect(trigger).toHaveAttribute('aria-expanded', 'false')
    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('supports defaultOpen', () => {
    render(
      <Popover.Root defaultOpen>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    expect(screen.getByRole('button', { name: 'Open' })).toHaveAttribute(
      'aria-expanded',
      'true'
    )
    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })

  it('calls onOpenChange when controlled', async () => {
    const handleOpenChange = vi.fn()

    render(
      <Popover.Root open={false} onOpenChange={handleOpenChange}>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    await userEvent.click(screen.getByRole('button', { name: 'Open' }))

    expect(handleOpenChange).toHaveBeenCalledWith(true)
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('closes when clicking outside', async () => {
    render(
      <div>
        <Popover.Root defaultOpen>
          <Popover.Trigger>Open</Popover.Trigger>
          <Popover.Content>Content</Popover.Content>
        </Popover.Root>
        <button type="button">Outside</button>
      </div>
    )

    expect(screen.getByRole('dialog')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Outside' }))

    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('does not close when clicking inside content', async () => {
    render(
      <Popover.Root defaultOpen>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>
          <button type="button">Inside</button>
        </Popover.Content>
      </Popover.Root>
    )

    await userEvent.click(screen.getByRole('button', { name: 'Inside' }))

    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })

  it('closes when Escape is pressed', async () => {
    render(
      <Popover.Root defaultOpen>
        <Popover.Trigger>Open</Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    await userEvent.keyboard('[Escape]')

    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('supports Trigger asChild', async () => {
    render(
      <Popover.Root>
        <Popover.Trigger asChild>
          <a href="/popover">Open</a>
        </Popover.Trigger>
        <Popover.Content>Content</Popover.Content>
      </Popover.Root>
    )

    expect(screen.queryByRole('button')).not.toBeInTheDocument()

    const trigger = screen.getByRole('link', { name: 'Open' })
    await userEvent.click(trigger)

    expect(trigger).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })

  it('forwards refs to trigger and content', async () => {
    const triggerRef = vi.fn()
    const contentRef = vi.fn()

    render(
      <Popover.Root>
        <Popover.Trigger ref={triggerRef}>Open</Popover.Trigger>
        <Popover.Content ref={contentRef}>Content</Popover.Content>
      </Popover.Root>
    )

    expect(triggerRef).toHaveBeenCalledWith(expect.any(HTMLButtonElement))

    await userEvent.click(screen.getByRole('button', { name: 'Open' }))

    expect(contentRef).toHaveBeenCalledWith(expect.any(HTMLDivElement))
  })
})
