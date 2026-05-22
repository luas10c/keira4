import { useRef, useEffect } from 'react'

import { OverlayScrollbars, ClickScrollPlugin } from 'overlayscrollbars'
import 'overlayscrollbars/overlayscrollbars.css'

import { Slot } from '#/components/atoms/slot'

import { cn } from '#/utils/cn'

// OverlayScrollbars plugins are registered globally and should only be set up once.
OverlayScrollbars.plugin(ClickScrollPlugin)

export function Scrollable({
  asChild,
  className,
  children,
  ...props
}: React.ComponentProps<'div'> & {
  asChild?: boolean
}) {
  const ref = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const element = ref.current

    if (!element) return

    const instance = OverlayScrollbars(element, {
      scrollbars: {
        theme: 'os-theme-dark scrollbar',
        autoHide: 'scroll',
        autoHideDelay: 900,
        autoHideSuspend: false,
        clickScroll: true
      },
      paddingAbsolute: false,
      overflow: {
        x: 'hidden'
      }
    })

    return () => {
      instance.destroy()
    }
  }, [])

  const Component = asChild ? Slot : 'div'

  return (
    <Component
      ref={ref}
      role="region"
      aria-label="Scrollable"
      className={cn('relative overflow-hidden pr-1', className)}
      {...props}
    >
      {children}
    </Component>
  )
}
