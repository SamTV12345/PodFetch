import { ChangeEventHandler, FC } from "react"

type CustomInputProps = {
    autoComplete?: string,
    className?: string,
    id?: string,
    name?: string,
    onChange?: ChangeEventHandler<HTMLInputElement>,
    placeholder?: string,
    required?: boolean,
    type?: string,
    value?: string | number
}

export const CustomInput: FC<CustomInputProps> = ({autoComplete, className = '', id, name, onChange, placeholder, required, type = 'text', value}) => {
    return (
        <input autoComplete={autoComplete} className={"bg-stone-100 px-4 py-2 rounded-lg text-sm text-stone-600 " + className} id={id} name={name} placeholder={placeholder} onChange={onChange} value={value} type={type} required={required} />
    )
}
