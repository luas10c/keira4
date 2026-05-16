import { isValidElement, forwardRef, Children, cloneElement } from 'react'

type AnyProps = Record<string, unknown>

/**
 * Reads the ref from an element in a way compatible with React 18 and 19.
 * React 19 warns when accessing `element.ref` directly.
 */
export function getElementRef<T = unknown>(
  element: React.ReactElement
): React.Ref<T> | undefined {
  const props = element.props as { ref?: React.Ref<T> }
  return (
    props.ref ?? (element as unknown as { ref?: React.Ref<T> }).ref ?? undefined
  )
}

export function composeRefs<T>(
  ...refs: (React.Ref<T> | undefined | null)[]
): React.RefCallback<T> {
  return (node: T | null) => {
    for (const ref of refs) {
      if (typeof ref === 'function') {
        ref(node)
      } else if (ref != null) {
        ;(ref as React.RefObject<T | null>).current = node
      }
    }
  }
}

/**
 * Merges Slot props with child props:
 * - Event handlers are composed (child runs first)
 * - `style` is shallowly merged (child overrides)
 * - `className` is concatenated
 * - All other props: child takes precedence
 */
export function mergeProps(
  slotProps: AnyProps,
  childProps: AnyProps
): AnyProps {
  const merged: AnyProps = { ...slotProps, ...childProps }

  for (const key in childProps) {
    const slotVal = slotProps[key]
    const childVal = childProps[key]

    if (
      /^on[A-Z]/.test(key) &&
      typeof slotVal === 'function' &&
      typeof childVal === 'function'
    ) {
      merged[key] = (...args: unknown[]) => {
        ;(childVal as (...a: unknown[]) => void)(...args)
        ;(slotVal as (...a: unknown[]) => void)(...args)
      }
      continue
    }

    if (key === 'style') {
      merged[key] = {
        ...(slotVal as React.CSSProperties | undefined),
        ...(childVal as React.CSSProperties | undefined)
      }
      continue
    }

    if (key === 'className') {
      merged[key] = [slotVal, childVal].filter(Boolean).join(' ') || undefined
      continue
    }
  }

  return merged
}

export interface SlotCloneProps {
  children?: React.ReactNode
  [key: string]: unknown
}

export const SlotClone = forwardRef<HTMLElement, SlotCloneProps>(
  ({ children, ...slotProps }, forwardedRef) => {
    if (!isValidElement(children)) {
      if (Children.count(children) > 1) {
        throw new Error(
          '[Slot] When not using <Slottable>, pass exactly one valid React child.'
        )
      }
      return null
    }

    const childRef = getElementRef<HTMLElement>(children)
    const composedRef =
      forwardedRef || childRef
        ? composeRefs<HTMLElement>(
            forwardedRef ?? undefined,
            childRef ?? undefined
          )
        : undefined

    return cloneElement(children, {
      ...mergeProps(slotProps, children.props as AnyProps),
      ...(composedRef ? { ref: composedRef } : {})
    } as React.HTMLAttributes<HTMLElement> & React.RefAttributes<HTMLElement>)
  }
)

SlotClone.displayName = 'SlotClone'
