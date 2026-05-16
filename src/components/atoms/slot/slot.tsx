import { isValidElement, forwardRef, Children, cloneElement } from 'react'

import { SlotClone } from './clone'
import { isSlottable } from './slottable'

export type SlotProps<T extends React.ElementType = 'span'> =
  React.ComponentPropsWithRef<T> & {
    children?: React.ReactNode
    asChild?: boolean
  }

/**
 * A `Slot` component compatible with @radix-ui/react-slot.
 *
 * Delegates rendering to its immediate child, merging props
 * (including event handlers and className) and composing refs.
 *
 * Supports the advanced `<Slottable>` API for composite layouts.
 *
 * @example Basic usage
 * <Slot onClick={handleClick} className="btn">
 *   <button>Click me</button>
 * </Slot>
 *
 * @example With Slottable (composite layout)
 * <Slot>
 *   <span className="icon">🔒</span>
 *   <Slottable>
 *     <button>Sign in</button>
 *   </Slottable>
 * </Slot>
 */
export const Slot = forwardRef<HTMLElement, SlotProps>(
  ({ children, ...slotProps }, forwardedRef) => {
    const childrenArray = Children.toArray(children)
    const slottable = childrenArray.find(isSlottable)

    if (!slottable) {
      return (
        <SlotClone ref={forwardedRef} {...slotProps}>
          {children}
        </SlotClone>
      )
    }

    const targetElement = slottable.props.children

    if (!isValidElement(targetElement)) {
      return null
    }

    const targetChildren = (
      targetElement.props as { children?: React.ReactNode }
    ).children

    const newChildren = childrenArray.map((child) =>
      child === slottable
        ? isValidElement(targetChildren)
          ? targetChildren
          : null
        : child
    )

    return (
      <SlotClone ref={forwardedRef} {...slotProps}>
        {cloneElement(targetElement, undefined, ...newChildren.filter(Boolean))}
      </SlotClone>
    )
  }
)

Slot.displayName = 'Slot'
