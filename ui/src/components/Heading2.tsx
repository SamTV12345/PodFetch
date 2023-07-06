import {FC} from "react"

type Heading2Props = {
  children: string,
  className?: string
}

export const Heading2:FC<Heading2Props> = ({children, className = ''}) => {
  return <h1 className={`font-bold !leading-tight text-xl xs:text-2xl text-stone-900 ${className}`}>{children}</h1>
}
