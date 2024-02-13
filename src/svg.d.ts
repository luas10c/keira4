declare module '*.svg' {
  export function ReactComponent(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    props: React.SVGProps<SVGElement>
  ): JSX.Element {
    return null
  }
}

declare module '*.svg?url' {
  const content: string
  export default content
}
