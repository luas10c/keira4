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
        'bg-cyan-500 text-white',
        'hover:bg-cyan-400',
        'active:bg-cyan-600'
      ],
      secondary: [
        'bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-zinc-100',
        'hover:bg-zinc-200 dark:hover:bg-zinc-700',
        'active:bg-zinc-300 dark:active:bg-zinc-600'
      ],
      outline: [
        'border border-zinc-300 bg-transparent text-zinc-900 dark:border-zinc-700 dark:text-zinc-100',
        'hover:bg-zinc-100 hover:text-zinc-900 dark:hover:bg-zinc-800 dark:hover:text-zinc-100',
        'active:bg-zinc-200 active:text-zinc-900 dark:active:bg-zinc-700 dark:active:text-zinc-100'
      ],
      ghost: [
        'bg-transparent text-zinc-100',
        'hover:bg-zinc-700',
        'active:bg-zinc-700'
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
