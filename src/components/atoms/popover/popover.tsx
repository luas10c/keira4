import {
  createContext,
  forwardRef,
  useContext,
  useEffect,
  useId,
  useRef,
  useState
} from 'react'
import {
  AnimatePresence,
  LazyMotion,
  domAnimation,
  m,
  type HTMLMotionProps
} from 'motion/react'

import { Slot } from '#/components/atoms/slot'
import { cn } from '#/utils/cn'

type PopoverCtx = {
  contentId: string
  open: boolean
  setOpen: (open: boolean) => void
  triggerRef: React.RefObject<HTMLElement | null>
  contentRef: React.RefObject<HTMLDivElement | null>
}

const PopoverContext = createContext<PopoverCtx | null>(null)

function usePopoverContext() {
  const ctx = useContext(PopoverContext)
  if (!ctx) throw new Error('<Popover.*> precisa estar dentro de <Popover.Root>')
  return ctx
}

function setRef<T>(ref: React.ForwardedRef<T>, value: T | null) {
  if (typeof ref === 'function') {
    ref(value)
  } else if (ref) {
    ref.current = value
  }
}

export type RootProps = {
  children: React.ReactNode
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
  className?: string
}

export function Root({
  children,
  open,
  defaultOpen = false,
  onOpenChange,
  className
}: RootProps) {
  const contentId = useId()
  const triggerRef = useRef<HTMLElement | null>(null)
  const contentRef = useRef<HTMLDivElement | null>(null)
  const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen)
  const resolvedOpen = open ?? uncontrolledOpen

  function setOpen(nextOpen: boolean) {
    if (open === undefined) setUncontrolledOpen(nextOpen)
    onOpenChange?.(nextOpen)
  }

  useEffect(() => {
    if (!resolvedOpen) return

    function handlePointerDown(e: PointerEvent) {
      const target = e.target as Node
      if (triggerRef.current?.contains(target)) return
      if (contentRef.current?.contains(target)) return

      setOpen(false)
    }

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }

    document.addEventListener('pointerdown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)

    return () => {
      document.removeEventListener('pointerdown', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [resolvedOpen])

  return (
    <PopoverContext.Provider
      value={{ contentId, open: resolvedOpen, setOpen, triggerRef, contentRef }}
    >
      <div className={cn('relative inline-block', className)}>{children}</div>
    </PopoverContext.Provider>
  )
}

export type TriggerProps = React.ComponentProps<'button'> & {
  asChild?: boolean
}

export const Trigger = forwardRef<HTMLElement, TriggerProps>(function Trigger(
  { asChild, children, onClick, ...rest },
  ref
) {
  const { contentId, open, setOpen, triggerRef } = usePopoverContext()
  const Component = asChild ? Slot : 'button'
  const type = !asChild && ({ type: 'button' } as const)

  function handleClick(e: React.MouseEvent<HTMLButtonElement>) {
    onClick?.(e)
    if (!e.defaultPrevented) setOpen(!open)
  }

  function handleRef(node: HTMLElement | null) {
    triggerRef.current = node
    setRef(ref, node)
  }

  return (
    <Component
      {...type}
      ref={handleRef}
      aria-expanded={open}
      aria-controls={contentId}
      data-state={open ? 'open' : 'closed'}
      onClick={handleClick}
      {...rest}
    >
      {children}
    </Component>
  )
})

export type ContentProps = HTMLMotionProps<'div'> & {
  align?: 'start' | 'center' | 'end'
}

const alignClass = {
  start: 'left-0',
  center: 'left-1/2 -translate-x-1/2',
  end: 'right-0'
}

export const Content = forwardRef<HTMLDivElement, ContentProps>(function Content(
  { align = 'start', className, children, ...rest },
  ref
) {
  const { contentId, open, contentRef } = usePopoverContext()

  function handleRef(node: HTMLDivElement | null) {
    contentRef.current = node
    setRef(ref, node)
  }

  return (
    <LazyMotion features={domAnimation}>
      <AnimatePresence>
        {open && (
          <m.div
            ref={handleRef}
            id={contentId}
            role="dialog"
            data-state="open"
            initial={{ opacity: 0, y: -2, scale: 0.995 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -2, scale: 0.995 }}
            transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}
            className={cn(
              'absolute top-full z-50 mt-2 min-w-48 rounded-md border border-[var(--input-border-color)]',
              'bg-[var(--input-background)] p-2 text-[var(--input-foreground)] shadow-lg outline-none',
              alignClass[align],
              className
            )}
            {...rest}
          >
            {children}
          </m.div>
        )}
      </AnimatePresence>
    </LazyMotion>
  )
})

export const Popover = { Root, Trigger, Content }
