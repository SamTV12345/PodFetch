import {ChangeEventHandler, FC, InputHTMLAttributes} from 'react'


export const CustomInput: FC<InputHTMLAttributes<HTMLInputElement>> = ({ autoComplete, onBlur, className = '', id, name, onChange, disabled, placeholder, required, type = 'text', value, ...props }) => {
    return (
        <input onBlur={onBlur} autoComplete={autoComplete} disabled={disabled} className={"bg-(--input-bg-color)" +
            " px-4 py-2 rounded-lg text-sm text-(--input-fg-color) placeholder:text-(--input-fg-color-disabled) " + className} id={id} name={name} placeholder={placeholder} onChange={onChange} value={value} type={type} required={required} {...props} />
    )
}
