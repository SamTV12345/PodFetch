import {ChangeEventHandler, FC, InputHTMLAttributes} from 'react'
import {LoadingSkeletonSpan} from "./ui/LoadingSkeletonSpan";


export const CustomInput: FC<InputHTMLAttributes<HTMLInputElement> &  {
    loading?: boolean
}> = ({ autoComplete, onBlur, className = '', id, name, onChange, disabled, placeholder, required, type = 'text', value, ...props }) => {
    if (props.loading) {
        return <LoadingSkeletonSpan height="30px" width="100px" text={""} loading={props.loading}/>
    }
    return (
        <input onBlur={onBlur} autoComplete={autoComplete} disabled={disabled} className={"bg-(--input-bg-color)" +
            " px-4 py-2 rounded-lg text-sm text-(--input-fg-color) placeholder:text-(--input-fg-color-disabled) " + className} id={id} name={name} placeholder={placeholder} onChange={onChange} value={value} type={type} required={required} {...props} />
    )
}
