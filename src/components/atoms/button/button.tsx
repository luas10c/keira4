import { tv, type VariantProps } from 'tailwind-variants'

import { Slot } from '#/components/atoms/slot'

const button = tv({
  base: [
    'inline-flex items-center justify-center gap-2',
    'rounded-md font-medium transition-colors',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
    'disabled:pointer-events-none disabled:opacity-60'
  ],
  variants: {
    variant: {
      primary: [
        'border border-[var(--button-primary-border-color)]',
        'bg-[var(--button-primary-background)] text-[var(--button-primary-foreground)]',
        'hover:bg-[var(--button-primary-hover-background)]',
        'active:bg-[var(--button-primary-active-background)]'
      ],
      secondary: [
        'border border-[var(--button-secondary-border-color)]',
        'bg-[var(--button-secondary-background)] text-[var(--button-secondary-foreground)]',
        'hover:bg-[var(--button-secondary-hover-background)]',
        'active:bg-[var(--button-secondary-active-background)]'
      ],
      outline: [
        'border border-[var(--outline-border-color)]',
        'bg-transparent text-[var(--outline-foreground)]',
        'hover:border-[var(--outline-hover-border-color)]',
        'hover:bg-[var(--outline-hover-background)]',
        'active:bg-[var(--outline-active-background)]'
      ],
      ghost: [
        'border border-transparent',
        'bg-transparent text-[var(--ghost-foreground)]',
        'hover:bg-[var(--ghost-hover-background)]',
        'active:bg-[var(--ghost-active-background)]'
      ]
    },
    size: {
      sm: 'h-8 px-3 text-sm',
      md: 'h-10 px-4 text-sm',
      lg: 'h-11 px-5 text-base'
    }
  },
  defaultVariants: {
    variant: 'ghost',
    size: 'md'
  }
})

type ButtonProps = React.ComponentProps<'button'> &
  VariantProps<typeof button> & {
    asChild?: boolean
  }

export function Button({
  children,
  asChild,
  className,
  variant,
  size,
  ...rest
}: ButtonProps) {
  const type = !asChild && ({ type: 'button' } as const)
  const Component = asChild ? Slot : 'button'

  return (
    <Component
      {...type}
      className={button({ variant, size, className })}
      {...rest}
    >
      {children}
    </Component>
  )
}
