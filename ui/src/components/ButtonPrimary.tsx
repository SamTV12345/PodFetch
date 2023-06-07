import {FC, ReactNode} from "react"

type ButtonPrimaryProps = {
    children: ReactNode,
    className?: string,
    disabled?: boolean,
    onClick?: () => void,
    type?: "button" | "submit" | "reset"
}

export const ButtonPrimary:FC<ButtonPrimaryProps> = ({children, className = '', disabled = false, onClick, type = 'button'}) => {
    return <button className={`bg-mustard-600 hover:bg-mustard-500 px-3 py-2 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_theme(colors.mustard.500)] text-sm text-white transition-colors disabled:opacity-50 disabled:shadow-none disabled:hover:bg-mustard-600 ${className}`} disabled={disabled} onClick={onClick} type={type}>{children}</button>
}
