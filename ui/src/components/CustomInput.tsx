import {FC, InputHTMLAttributes, useEffect, useRef, useState} from 'react'
import {LoadingSkeletonSpan} from "./ui/LoadingSkeletonSpan";


export const CustomInput: FC<InputHTMLAttributes<HTMLInputElement> &  {
    loading?: boolean
}> = ({ autoComplete, onBlur, className = '', id, name, onChange, disabled, placeholder, required, type = 'text', value, ...props }) => {
    // Some callers keep the value in an external store (e.g. the react-query
    // cache via setQueryData) that only notifies subscribers asynchronously.
    // React then reverts the DOM value right after each input event, which
    // cancels an active IME composition — Chinese/Japanese/Korean typing
    // becomes unusable. Mirroring the value locally keeps the DOM update
    // synchronous; external changes are applied while no composition runs.
    const [localValue, setLocalValue] = useState(value)
    const isComposing = useRef(false)

    useEffect(() => {
        if (!isComposing.current) {
            setLocalValue(value)
        }
    }, [value])

    if (props.loading) {
        return <LoadingSkeletonSpan height="30px" width="100px" text={""} loading={props.loading}/>
    }
    return (
        <input onBlur={onBlur} autoComplete={autoComplete} disabled={disabled} className={"ui-input-surface" +
            " px-4 py-2 rounded-lg text-sm ui-input-text placeholder:ui-input-text-disabled " + className} id={id} name={name} placeholder={placeholder} onChange={(e) => {
                setLocalValue(e.target.value)
                onChange?.(e)
            }} value={localValue ?? ''} type={type} required={required} {...props}
            onCompositionStart={() => { isComposing.current = true }}
            onCompositionEnd={() => { isComposing.current = false }} />
    )
}
