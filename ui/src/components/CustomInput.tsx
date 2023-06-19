import { ChangeEventHandler, FC } from "react"

type CustomInputProps = {
    className?: string,
    id?: string,
    name?: string,
    onChange?: ChangeEventHandler<HTMLInputElement>,
    placeholder?: string,
    type?: string,
    value?: string | number
}

export const CustomInput: FC<CustomInputProps> = ({className = '', id, name, onChange, placeholder, type = 'text', value}) => {
    return (
        <input className={"bg-stone-100 px-4 py-2 rounded-lg text-sm text-stone-600 " + className} id={id} name={name} placeholder={placeholder} onChange={onChange} value={value} type={type}/>
    )
}
