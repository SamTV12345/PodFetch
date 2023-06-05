import {ChangeEventHandler, FC} from "react"

type InputProps = {
  className?: string,
  type?: string,
  placeholder?: string,
  value?: string,
  onChange?: ChangeEventHandler<HTMLInputElement>
}

export const Input:FC<InputProps> = ({className, type = "text", placeholder, value, onChange}) => {
  return <input type={type} placeholder={placeholder} value={value} onChange={onChange}
  className={"bg-stone-100 w-full pl-10 pr-4 py-2 rounded-lg text-sm text-stone-600 " + className}/>
}
