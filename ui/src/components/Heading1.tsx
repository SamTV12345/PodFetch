import {FC} from "react"

type Heading1Props = {
  children: string,
  className?: string
}

export const Heading1:FC<Heading1Props> = ({children, className}) => {
  return <h1 className={`font-bold leading-none text-4xl text-stone-900 ${className}`}>{children}</h1>
}
