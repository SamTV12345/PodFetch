import {FC, ReactNode} from "react"

type ButtonSecondaryProps = {
    children: ReactNode,
    className?: string,
    disabled?: boolean,
    onClick: () => void,
    type?: "button" | "submit" | "reset"
}

export const ButtonSecondary:FC<ButtonSecondaryProps> = ({children, className = '', disabled = false, onClick, type}) => {
    return <button className={`border border-mustard-600 px-3 py-2 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.1)] hover:shadow-[0_2px_16px_theme(colors.mustard.300)] text-sm text-mustard-600 transition-shadow whitespace-nowrap disabled:opacity-50 disabled:shadow-none ${className}`} disabled={disabled} onClick={onClick} type={type}>{children}</button>
}
