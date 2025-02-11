import { FC, ReactNode } from 'react'

type CustomButtonSecondaryProps = {
    children: ReactNode,
    className?: string,
    disabled?: boolean,
    onClick?: () => void,
    type?: "button" | "submit" | "reset"
}

export const CustomButtonSecondary: FC<CustomButtonSecondaryProps> = ({ children, className = '', disabled = false, onClick, type }) => {
    return (
        <button className={`border border-(--acent-color) leading-none px-4 py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.1)] hover:shadow-[0_2px_16px_var(--accent-color-hover)] text-left text-sm text-(--accent-color) transition-shadow disabled:opacity-50 disabled:shadow-none ${className}`} disabled={disabled} onClick={onClick} type={type}>{children}</button>
    )
}
