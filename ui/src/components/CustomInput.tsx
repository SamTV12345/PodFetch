import {FC, InputHTMLAttributes, useEffect, useRef, useState} from 'react'
import {LoadingSkeletonSpan} from "./ui/LoadingSkeletonSpan";


export const CustomInput: FC<InputHTMLAttributes<HTMLInputElement> &  {
    loading?: boolean
}> = ({ autoComplete, onBlur, onCompositionEnd, onCompositionStart, className = '', id, name, onChange, disabled, loading, placeholder, required, type = 'text', value, ...props }) => {
    const [localValue, setLocalValue] = useState(value)
    const localValueRef = useRef(value)
    const isComposing = useRef(false)
    const inputSequence = useRef(0)
    const pendingExternalValue = useRef<{ value: typeof value, inputSequence: number } | null>(null)

    useEffect(() => {
        if (isComposing.current) {
            pendingExternalValue.current = {value, inputSequence: inputSequence.current}
            return
        }

        localValueRef.current = value
        setLocalValue(value)
    }, [value])

    if (loading) {
        return <LoadingSkeletonSpan height="30px" width="100px" text={""} loading={loading}/>
    }

    return (
        <input onBlur={onBlur} autoComplete={autoComplete} disabled={disabled} className={"ui-input-surface" +
            " px-4 py-2 rounded-lg text-sm ui-input-text placeholder:ui-input-text-disabled " + className} id={id} name={name} placeholder={placeholder} onChange={(e) => {
                inputSequence.current += 1
                localValueRef.current = e.target.value
                setLocalValue(e.target.value)
                onChange?.(e)
            }} value={localValue ?? ''} type={type} required={required} {...props}
            onCompositionStart={(e) => {
                isComposing.current = true
                pendingExternalValue.current = null
                onCompositionStart?.(e)
            }}
            onCompositionEnd={(e) => {
                isComposing.current = false
                onCompositionEnd?.(e)

                // React may deliver the final input event immediately after
                // compositionend. Reconcile a genuine external update in a
                // microtask, but never overwrite that final user input with
                // an older cache echo.
                const pending = pendingExternalValue.current
                if (pending) {
                    queueMicrotask(() => {
                        if (inputSequence.current !== pending.inputSequence || isComposing.current) {
                            return
                        }
                        if (!Object.is(pending.value, localValueRef.current)) {
                            localValueRef.current = pending.value
                            setLocalValue(pending.value)
                        }
                        pendingExternalValue.current = null
                    })
                }
            }} />
    )
}
