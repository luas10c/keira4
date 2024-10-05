declare module '*.svg' {
  export function ReactComponent(
    props: React.SVGProps<SVGElement>
  ): JSX.Element {
    return null
  }
}

declare module '*.svg?url' {
  const content: string
  export default content
}
