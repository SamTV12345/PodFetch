import {FC} from "react"

type Heading3Props = {
  children: string,
  className?: string
}

export const Heading3:FC<Heading3Props> = ({children, className = ''}) => {
  return <h1 className={`font-bold !leading-tight text-stone-900 ${className}`}>{children}</h1>
}
