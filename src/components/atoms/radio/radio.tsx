import { createContext, forwardRef, useContext, useId, useState } from 'react'

import { cn } from '#/utils/cn'

type RadioSize = 'sm' | 'md' | 'lg'

type RadioRootCtx = {
  radioId: string
  descriptionId: string
  checked: boolean
  disabled: boolean
  size: RadioSize
}

type RadioGroupCtx = {
  name: string
  value?: string
  onChange: (value: string) => void
  disabled: boolean
}

const RadioRootContext = createContext<RadioRootCtx | null>(null)
const RadioGroupContext = createContext<RadioGroupCtx | null>(null)

function useRadioRootContext() {
  const ctx = useContext(RadioRootContext)
  if (!ctx) throw new Error('<Radio.*> precisa estar dentro de <Radio.Root>')
  return ctx
}

const controlSize = {
  sm: 'size-4',
  md: 'size-5',
  lg: 'size-6'
}

const indicatorSize = {
  sm: 'size-1.5',
  md: 'size-2',
  lg: 'size-2.5'
}

export type RootProps = {
  'children'?: React.ReactNode
  'checked'?: boolean
  'defaultChecked'?: boolean
  'onCheckedChange'?: (checked: boolean) => void
  'disabled'?: boolean
  'required'?: boolean
  'name'?: string
  'value'?: string
  'className'?: string
  'aria-label'?: string
  'aria-invalid'?: boolean
  'size'?: RadioSize
}

export const Root = forwardRef<HTMLDivElement, RootProps>(function Root(
  {
    children,
    checked,
    defaultChecked,
    onCheckedChange,
    disabled = false,
    required = false,
    name,
    value = 'on',
    className,
    size = 'sm',
    'aria-label': ariaLabelProp,
    'aria-invalid': ariaInvalidProp
  },
  ref
) {
  const uid = useId()
  const radioId = `${uid}-radio`
  const descriptionId = `${uid}-description`
  const group = useContext(RadioGroupContext)
  const [uncontrolledChecked, setUncontrolledChecked] = useState(
    defaultChecked ?? false
  )
  const isDisabled = disabled || (group?.disabled ?? false)
  const inputName = name ?? group?.name
  const label = ariaLabelProp
  const invalid = ariaInvalidProp

  const isChecked = group
    ? group.value === value
    : (checked ?? uncontrolledChecked)

  function handleSelect() {
    if (isDisabled || isChecked) return

    if (group) {
      group.onChange(value)
      onCheckedChange?.(true)
      return
    }

    if (checked === undefined) {
      setUncontrolledChecked(true)
    }

    onCheckedChange?.(true)
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLButtonElement>) {
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault()
      handleSelect()
    }
  }

  return (
    <RadioRootContext.Provider
      value={{
        radioId,
        descriptionId,
        checked: isChecked,
        disabled: isDisabled,
        size
      }}
    >
      <div
        ref={ref}
        data-disabled={isDisabled || undefined}
        data-invalid={invalid || undefined}
        className={cn('flex items-center gap-3', className)}
      >
        <button
          type="button"
          role="radio"
          id={radioId}
          aria-checked={isChecked}
          aria-disabled={isDisabled || undefined}
          aria-label={label}
          aria-describedby={descriptionId}
          data-state={isChecked ? 'checked' : 'unchecked'}
          data-disabled={isDisabled || undefined}
          tabIndex={isDisabled ? -1 : 0}
          onClick={handleSelect}
          onKeyDown={handleKeyDown}
          className={cn(
            'inline-flex shrink-0 items-center justify-center rounded-full',
            'border transition-colors outline-none select-none',
            'focus-visible:ring-1 focus-visible:ring-[var(--input-focus-border-color)]',
            controlSize[size],
            isDisabled
              ? 'cursor-not-allowed border-[var(--radio-border-color)] bg-[var(--radio-background)] text-[var(--input-placeholder)] opacity-60'
              : isChecked
                ? 'cursor-pointer border-[var(--radio-active-border-color)] bg-[var(--radio-active-background)] text-[var(--radio-active-foreground)]'
                : cn(
                    'cursor-pointer border-[var(--radio-border-color)] bg-[var(--radio-background)] text-[var(--radio-foreground)]',
                    invalid && 'border-red-500 hover:border-red-400'
                  )
          )}
        >
          <Indicator />
        </button>

        {children && <div className="flex flex-col gap-1">{children}</div>}

        {inputName && (
          <input
            type="radio"
            name={inputName}
            value={value}
            checked={isChecked}
            disabled={isDisabled}
            required={required}
            aria-invalid={invalid}
            aria-hidden
            tabIndex={-1}
            onChange={() => {}}
            className="sr-only"
          />
        )}
      </div>
    </RadioRootContext.Provider>
  )
})

export type IndicatorProps = React.ComponentProps<'span'>

export const Indicator = forwardRef<HTMLSpanElement, IndicatorProps>(
  function Indicator({ className, ...props }, ref) {
    const { checked, disabled, size } = useRadioRootContext()

    return (
      <span
        ref={ref}
        data-state={checked ? 'checked' : 'unchecked'}
        className={cn(
          'block rounded-full transition-transform duration-150 ease-out',
          indicatorSize[size],
          checked ? 'scale-100 opacity-100' : 'scale-0 opacity-0',
          disabled ? 'bg-[var(--input-placeholder)]' : 'bg-current',
          className
        )}
        {...props}
      />
    )
  }
)

export type LabelProps = {
  children: React.ReactNode
  className?: string
}

export function Label({ children, className }: LabelProps) {
  const { radioId, disabled } = useRadioRootContext()

  return (
    <label
      htmlFor={radioId}
      data-disabled={disabled || undefined}
      className={cn(
        'text-sm leading-normal select-none',
        disabled
          ? 'cursor-not-allowed text-[var(--input-placeholder)]'
          : 'cursor-pointer text-[var(--workbench-foreground)]',
        className
      )}
    >
      {children}
    </label>
  )
}

export type DescriptionProps = {
  children: React.ReactNode
  className?: string
}

export function Description({ children, className }: DescriptionProps) {
  const { descriptionId } = useRadioRootContext()

  return (
    <p
      id={descriptionId}
      className={cn(
        'text-xs leading-relaxed text-[var(--input-placeholder)]',
        className
      )}
    >
      {children}
    </p>
  )
}

export type GroupProps = {
  children: React.ReactNode
  name: string
  value?: string
  onValueChange: (value: string) => void
  disabled?: boolean
  className?: string
}

export function Group({
  children,
  name,
  value,
  onValueChange,
  disabled = false,
  className
}: GroupProps) {
  return (
    <RadioGroupContext.Provider
      value={{ name, value, onChange: onValueChange, disabled }}
    >
      <div
        role="radiogroup"
        aria-label={name}
        className={cn('flex flex-col gap-3', className)}
      >
        {children}
      </div>
    </RadioGroupContext.Provider>
  )
}
