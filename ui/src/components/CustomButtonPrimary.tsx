import { FC, ReactNode } from 'react'

type CustomButtonPrimaryProps = {
    children: ReactNode,
    className?: string,
    disabled?: boolean,
    onClick?: () => void,
    type?: "button" | "submit" | "reset"
}

export const CustomButtonPrimary: FC<CustomButtonPrimaryProps> = ({ children, className = '', disabled = false, onClick, type = 'button' }) => {
    return (
        <button className={`bg-[--accent-color] hover:bg-[--accent-color-hover] leading-none px-4 py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_var(--accent-color-hover)] text-left text-sm text-[--bg-color] transition disabled:opacity-50 disabled:shadow-none disabled:hover:bg-[--accent-color] ${className}`} disabled={disabled} onClick={onClick} type={type}>{children}</button>
    )
}
