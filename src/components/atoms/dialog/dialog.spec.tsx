import { describe, expect, it, vi } from 'vitest'
import {
  render,
  screen,
  waitForElementToBeRemoved
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import * as Dialog from '.'

describe('Dialog', () => {
  function Example(props?: Partial<Dialog.RootProps>) {
    return (
      <Dialog.Root {...props}>
        <Dialog.Trigger>Open</Dialog.Trigger>
        <Dialog.Overlay />
        <Dialog.Content>
          <div className="p-4">
            <Dialog.Title>Settings</Dialog.Title>
            <Dialog.Description>Manage your preferences</Dialog.Description>
            <Dialog.Close>Close</Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Root>
    )
  }

  it('renders closed by default', () => {
    render(<Example />)

    expect(screen.getByRole('button', { name: 'Open' })).toBeInTheDocument()
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('opens and closes from trigger and close button', async () => {
    render(<Example />)

    await userEvent.click(screen.getByRole('button', { name: 'Open' }))

    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByRole('dialog')).toHaveAttribute('aria-modal', 'true')

    await userEvent.click(screen.getByRole('button', { name: 'Close' }))

    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('supports defaultOpen', () => {
    render(<Example defaultOpen />)

    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('Settings')).toBeInTheDocument()
  })

  it('calls onOpenChange when controlled', async () => {
    const handleOpenChange = vi.fn()

    render(<Example open={false} onOpenChange={handleOpenChange} />)

    await userEvent.click(screen.getByRole('button', { name: 'Open' }))

    expect(handleOpenChange).toHaveBeenCalledWith(true)
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('closes when clicking the overlay', async () => {
    render(<Example defaultOpen />)

    await userEvent.click(
      document.querySelector('[data-slot="dialog-overlay"]') as Element
    )

    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('does not close when clicking inside content', async () => {
    render(<Example defaultOpen />)

    await userEvent.click(screen.getByText('Settings'))

    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })

  it('closes when Escape is pressed', async () => {
    render(<Example defaultOpen />)

    await userEvent.keyboard('[Escape]')

    await waitForElementToBeRemoved(() => screen.queryByRole('dialog'))
  })

  it('supports Trigger asChild', async () => {
    render(
      <Dialog.Root>
        <Dialog.Trigger asChild>
          <a href="/dialog">Open</a>
        </Dialog.Trigger>
        <Dialog.Overlay />
        <Dialog.Content>
          <Dialog.Title>Settings</Dialog.Title>
        </Dialog.Content>
      </Dialog.Root>
    )

    expect(screen.queryByRole('button')).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('link', { name: 'Open' }))

    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })

  it('wires title and description ids for accessibility', async () => {
    render(<Example defaultOpen />)

    const dialog = screen.getByRole('dialog')
    const title = screen.getByText('Settings')
    const description = screen.getByText('Manage your preferences')

    expect(dialog).toHaveAttribute('aria-labelledby', title.id)
    expect(dialog).toHaveAttribute('aria-describedby', description.id)
  })

  it('forwards refs to content and close trigger', async () => {
    const contentRef = vi.fn()
    const closeRef = vi.fn()

    render(
      <Dialog.Root defaultOpen>
        <Dialog.Overlay />
        <Dialog.Content ref={contentRef}>
          <Dialog.Title>Settings</Dialog.Title>
          <Dialog.Close ref={closeRef}>Close</Dialog.Close>
        </Dialog.Content>
      </Dialog.Root>
    )

    expect(contentRef).toHaveBeenCalledWith(expect.any(HTMLDivElement))
    expect(closeRef).toHaveBeenCalledWith(expect.any(HTMLButtonElement))
  })
})
