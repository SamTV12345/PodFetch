import {FC} from "react"

type Heading1Props = {
  children: string,
  className?: string
}

export const Heading1:FC<Heading1Props> = ({children, className}) => {
  return <h1 className={`text-4xl font-bold leading-none text-stone-900 ${className}`}>{children}</h1>
}
