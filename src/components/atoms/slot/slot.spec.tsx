import { vi, describe, it, expect } from 'vitest'
import { screen, render } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createRef } from 'react'

import { Slot, Slottable } from '.'

describe('Slot', () => {
  describe('basic rendering', () => {
    it('should render the child element in place of the Slot', () => {
      render(
        <Slot>
          <button>Click</button>
        </Slot>
      )

      expect(screen.getByRole('button', { name: /Click/i })).toBeInTheDocument()
    })

    it('should return null when there is no valid child', () => {
      const { container } = render(<Slot>{null}</Slot>)

      expect(container).toBeEmptyDOMElement()
    })

    it('should throw when more than one child is passed without Slottable', () => {
      expect(() =>
        render(
          <Slot>
            <span>A</span>
            <span>B</span>
          </Slot>
        )
      ).toThrow('[Slot]')
    })
  })

  describe('props merging', () => {
    it('should forward Slot className to the child', () => {
      render(
        <Slot className="slot-class">
          <button>Label</button>
        </Slot>
      )

      expect(screen.getByRole('button')).toHaveClass('slot-class')
    })

    it('should concatenate Slot className with child className', () => {
      render(
        <Slot className="slot-class">
          <button className="child-class">Label</button>
        </Slot>
      )

      const button = screen.getByRole('button')

      expect(button).toHaveClass('slot-class')
      expect(button).toHaveClass('child-class')
    })

    it('should merge Slot style with child style, child takes precedence', () => {
      render(
        <Slot style={{ color: 'red', fontWeight: 'bold' }}>
          <button style={{ color: 'blue' }}>Label</button>
        </Slot>
      )

      expect(screen.getByRole('button')).toHaveStyle({
        color: 'blue',
        fontWeight: 'bold'
      })
    })

    it('should forward arbitrary attributes from Slot to the child', () => {
      render(
        <Slot data-testid="slot-attr">
          <button>Label</button>
        </Slot>
      )

      expect(screen.getByTestId('slot-attr')).toBeInTheDocument()
    })

    it('should give child props precedence over Slot props', () => {
      render(
        <Slot id="slot-id">
          <button id="child-id">Label</button>
        </Slot>
      )

      expect(screen.getByRole('button').id).toBe('child-id')
    })

    it('should not set className when both Slot and child className are empty strings', () => {
      render(
        <Slot className="">
          <button className="">Label</button>
        </Slot>
      )

      expect(screen.getByRole('button')).not.toHaveAttribute('class')
    })
  })

  describe('event handler composition', () => {
    it('should call both child and Slot onClick handlers', async () => {
      const slotHandle = vi.fn()
      const childHandle = vi.fn()

      render(
        <Slot onClick={slotHandle}>
          <button onClick={childHandle}>Label</button>
        </Slot>
      )

      await userEvent.click(screen.getByRole('button'))

      expect(slotHandle).toHaveBeenCalledTimes(1)
      expect(childHandle).toHaveBeenCalledTimes(1)
    })

    it('should call child handler before Slot handler', async () => {
      const order: string[] = []

      render(
        <Slot onClick={() => order.push('slot')}>
          <button onClick={() => order.push('child')}>Label</button>
        </Slot>
      )

      await userEvent.click(screen.getByRole('button'))

      expect(order).toEqual(['child', 'slot'])
    })

    it('should work when only the Slot has a handler', async () => {
      const handle = vi.fn()

      render(
        <Slot onClick={handle}>
          <button>Label</button>
        </Slot>
      )

      await userEvent.click(screen.getByRole('button'))

      expect(handle).toHaveBeenCalledTimes(1)
    })

    it('should work when only the child has a handler', async () => {
      const handle = vi.fn()

      render(
        <Slot>
          <button onClick={handle}>Label</button>
        </Slot>
      )

      await userEvent.click(screen.getByRole('button'))

      expect(handle).toHaveBeenCalledTimes(1)
    })
  })

  describe('ref composition', () => {
    it('should forward the Slot ref to the child element', () => {
      const ref = createRef<HTMLButtonElement>()

      render(
        <Slot ref={ref as React.Ref<HTMLElement>}>
          <button>Label</button>
        </Slot>
      )

      expect(ref.current).toBeInstanceOf(HTMLButtonElement)
    })

    it('should compose Slot ref and child ref pointing to the same node', () => {
      const slotRef = createRef<HTMLButtonElement>()
      const childRef = createRef<HTMLButtonElement>()

      render(
        <Slot ref={slotRef as React.Ref<HTMLElement>}>
          <button ref={childRef}>Label</button>
        </Slot>
      )

      expect(slotRef.current).toBeInstanceOf(HTMLButtonElement)
      expect(childRef.current).toBeInstanceOf(HTMLButtonElement)
      expect(slotRef.current).toBe(childRef.current)
    })

    it('should support callback refs', () => {
      const callbackRef = vi.fn()

      render(
        <Slot ref={callbackRef}>
          <button>Label</button>
        </Slot>
      )

      expect(callbackRef).toHaveBeenCalledWith(expect.any(HTMLButtonElement))
    })

    it('should resolve child ref even when Slot has no ref', () => {
      const childRef = createRef<HTMLButtonElement>()

      render(
        <Slot>
          <button ref={childRef}>Label</button>
        </Slot>
      )

      expect(childRef.current).toBeInstanceOf(HTMLButtonElement)
    })
  })
})

describe('Slot with Slottable', () => {
  it('should render the element inside Slottable as the root', () => {
    render(
      <Slot>
        <Slottable>
          <a href="/home">Home</a>
        </Slottable>
      </Slot>
    )

    expect(screen.getByRole('link', { name: /Home/i })).toBeInTheDocument()
  })

  it('should keep Slottable siblings as children of the target element', () => {
    render(
      <Slot>
        <span data-testid="before">Before</span>
        <Slottable>
          <a href="/home">Home</a>
        </Slottable>
        <span data-testid="after">After</span>
      </Slot>
    )

    const link = screen.getByRole('link')

    expect(link).toContainElement(screen.getByTestId('before'))
    expect(link).toContainElement(screen.getByTestId('after'))
  })

  it('should merge Slot props with the element inside Slottable', () => {
    render(
      <Slot className="slot-class">
        <Slottable>
          <a href="/home" className="child-class">
            Home
          </a>
        </Slottable>
      </Slot>
    )

    const link = screen.getByRole('link')

    expect(link).toHaveClass('slot-class')
    expect(link).toHaveClass('child-class')
  })

  it('should return null when the Slottable child is not a valid element', () => {
    const { container } = render(
      <Slot>
        <Slottable>{null}</Slottable>
      </Slot>
    )

    expect(container).toBeEmptyDOMElement()
  })

  it('should compose Slot handlers with the element inside Slottable', async () => {
    const slotHandle = vi.fn()
    const childHandle = vi.fn()

    render(
      <Slot onClick={slotHandle}>
        <Slottable>
          <button onClick={childHandle}>Label</button>
        </Slottable>
      </Slot>
    )

    await userEvent.click(screen.getByRole('button'))

    expect(slotHandle).toHaveBeenCalledTimes(1)
    expect(childHandle).toHaveBeenCalledTimes(1)
  })

  it('should preserve target element own children when Slottable has no siblings', () => {
    render(
      <Slot>
        <Slottable>
          <a href="/home">
            <span data-testid="inner">Inner</span>
          </a>
        </Slottable>
      </Slot>
    )

    expect(screen.getByTestId('inner')).toBeInTheDocument()
  })
})

describe('Slottable', () => {
  it('should render its children without a wrapper element', () => {
    render(
      <Slottable>
        <span>Text</span>
      </Slottable>
    )

    expect(screen.getByText('Text').tagName).toBe('SPAN')
  })
})
