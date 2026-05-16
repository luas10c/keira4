import { isValidElement } from 'react'

export type SlottableProps = React.PropsWithChildren

export function Slottable({ children }: SlottableProps): React.ReactElement {
  return <>{children}</>
}

export function isSlottable(
  child: React.ReactNode
): child is React.ReactElement<SlottableProps> {
  return isValidElement(child) && child.type === Slottable
}
