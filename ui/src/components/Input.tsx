import {ChangeEventHandler, FC} from "react"

type InputProps = {
  className?: string,
  type?: string,
  placeholder?: string,
  name?: string,
  value?: string,
  onChange?: ChangeEventHandler<HTMLInputElement>
}

export const Input:FC<InputProps> = ({className = '', type = 'text', placeholder, name, value, onChange}) => {
  return <input type={type} placeholder={placeholder} name={name} value={value} onChange={onChange}
  className={"bg-stone-100 w-full px-4 py-2 rounded-lg text-sm text-stone-600 " + className}/>
}
