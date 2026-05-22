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

type DialogCtx = {
  titleId: string
  descriptionId: string
  open: boolean
  setOpen: (open: boolean) => void
  contentRef: React.RefObject<HTMLDivElement | null>
}

const DialogContext = createContext<DialogCtx | null>(null)

function useDialogContext() {
  const ctx = useContext(DialogContext)
  if (!ctx) throw new Error('<Dialog.*> precisa estar dentro de <Dialog.Root>')
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
}

export function Root({
  children,
  open,
  defaultOpen = false,
  onOpenChange
}: RootProps) {
  const titleId = useId()
  const descriptionId = useId()
  const contentRef = useRef<HTMLDivElement | null>(null)
  const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen)
  const resolvedOpen = open ?? uncontrolledOpen

  function setOpen(nextOpen: boolean) {
    if (open === undefined) setUncontrolledOpen(nextOpen)
    onOpenChange?.(nextOpen)
  }

  useEffect(() => {
    if (!resolvedOpen) return

    const previousOverflow = document.body.style.overflow
    document.body.style.overflow = 'hidden'

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }

    document.addEventListener('keydown', handleKeyDown)

    return () => {
      document.body.style.overflow = previousOverflow
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [resolvedOpen])

  return (
    <DialogContext.Provider
      value={{
        titleId,
        descriptionId,
        open: resolvedOpen,
        setOpen,
        contentRef
      }}
    >
      {children}
    </DialogContext.Provider>
  )
}

export type TriggerProps = React.ComponentProps<'button'> & {
  asChild?: boolean
}

export const Trigger = forwardRef<HTMLElement, TriggerProps>(function Trigger(
  { asChild, children, onClick, ...rest },
  ref
) {
  const { open, setOpen } = useDialogContext()
  const Component = asChild ? Slot : 'button'
  const type = !asChild && ({ type: 'button' } as const)

  function handleClick(e: React.MouseEvent<HTMLButtonElement>) {
    onClick?.(e)
    if (!e.defaultPrevented) setOpen(!open)
  }

  function handleRef(node: HTMLElement | null) {
    setRef(ref, node)
  }

  return (
    <Component {...type} ref={handleRef} onClick={handleClick} {...rest}>
      {children}
    </Component>
  )
})

export type OverlayProps = HTMLMotionProps<'div'>

export const Overlay = forwardRef<HTMLDivElement, OverlayProps>(
  function Overlay({ className, onClick, ...rest }, ref) {
    const { open, setOpen, contentRef } = useDialogContext()

    if (!open) return null

    function handleClick(e: React.MouseEvent<HTMLDivElement>) {
      onClick?.(e)
      if (e.defaultPrevented) return

      const target = e.target as Node
      if (contentRef.current?.contains(target)) return

      setOpen(false)
    }

    return (
      <LazyMotion features={domAnimation}>
        <AnimatePresence>
          {open && (
            <m.div
              ref={ref}
              data-slot="dialog-overlay"
              data-state="open"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}
              className={cn(
                'fixed inset-0 z-50 bg-black/55 backdrop-blur-[1px]',
                className
              )}
              onClick={handleClick}
              {...rest}
            />
          )}
        </AnimatePresence>
      </LazyMotion>
    )
  }
)

export type ContentProps = HTMLMotionProps<'div'>

export const Content = forwardRef<HTMLDivElement, ContentProps>(
  function Content({ className, children, ...rest }, ref) {
    const { titleId, descriptionId, open, contentRef } = useDialogContext()

    function handleRef(node: HTMLDivElement | null) {
      contentRef.current = node
      setRef(ref, node)
    }

    useEffect(() => {
      if (!open) return
      contentRef.current?.focus()
    }, [open])

    return (
      <LazyMotion features={domAnimation}>
        <AnimatePresence>
          {open && (
            <div className="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-4">
              <m.div
                ref={handleRef}
                data-slot="dialog-content"
                role="dialog"
                aria-modal="true"
                aria-labelledby={titleId}
                aria-describedby={descriptionId}
                tabIndex={-1}
                data-state="open"
                initial={{ opacity: 0, y: 4, scale: 0.995 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 4, scale: 0.995 }}
                transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}
                className={cn(
                  'pointer-events-auto relative z-10 w-full max-w-lg rounded-xl border border-[var(--input-border-color)]',
                  'bg-[var(--workbench-background)] text-[var(--workbench-foreground)] shadow-2xl outline-none',
                  className
                )}
                {...rest}
              >
                {children}
              </m.div>
            </div>
          )}
        </AnimatePresence>
      </LazyMotion>
    )
  }
)

export type TitleProps = React.ComponentProps<'h2'>

export function Title({ className, ...rest }: TitleProps) {
  const { titleId } = useDialogContext()

  return (
    <h2
      id={titleId}
      className={cn('text-lg font-medium', className)}
      {...rest}
    />
  )
}

export type DescriptionProps = React.ComponentProps<'p'>

export function Description({ className, ...rest }: DescriptionProps) {
  const { descriptionId } = useDialogContext()

  return (
    <p
      id={descriptionId}
      className={cn('text-sm text-[var(--input-placeholder)]', className)}
      {...rest}
    />
  )
}

export type CloseProps = React.ComponentProps<'button'> & {
  asChild?: boolean
}

export const Close = forwardRef<HTMLElement, CloseProps>(function Close(
  { asChild, children, onClick, ...rest },
  ref
) {
  const { setOpen } = useDialogContext()
  const Component = asChild ? Slot : 'button'
  const type = !asChild && ({ type: 'button' } as const)

  function handleClick(e: React.MouseEvent<HTMLButtonElement>) {
    onClick?.(e)
    if (!e.defaultPrevented) setOpen(false)
  }

  function handleRef(node: HTMLElement | null) {
    setRef(ref, node)
  }

  return (
    <Component {...type} ref={handleRef} onClick={handleClick} {...rest}>
      {children}
    </Component>
  )
})

export const Dialog = {
  Root,
  Trigger,
  Overlay,
  Content,
  Title,
  Description,
  Close
}
