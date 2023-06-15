import {ChangeEventHandler, FC} from "react"

type CustomInputProps = {
  className?: string,
  type?: string,
  placeholder?: string,
  name?: string,
  value?: string | number,
  onChange?: ChangeEventHandler<HTMLInputElement>
}

export const CustomInput:FC<CustomInputProps> = ({className = '', type = 'text', placeholder, name, value, onChange}) => {
  return <input type={type} placeholder={placeholder} name={name} value={value} onChange={onChange}
  className={"bg-stone-100 px-4 py-2 rounded-lg text-sm text-stone-600 " + className}/>
}
