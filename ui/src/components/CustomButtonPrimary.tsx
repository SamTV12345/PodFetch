import { FC, ReactNode } from 'react'
import {LoadingSkeletonSpan} from "./ui/LoadingSkeletonSpan";

type CustomButtonPrimaryProps = {
    children: ReactNode,
    className?: string,
    disabled?: boolean,
    onClick?: () => void,
    type?: "button" | "submit" | "reset"
    loading?: boolean
}

export const CustomButtonPrimary: FC<CustomButtonPrimaryProps> = ({ children, className = '', disabled = false, onClick, type = 'button', loading }) => {
    return (
        <button className={`ui-bg-accent hover:ui-bg-accent-hover leading-none px-4 py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_var(--accent-color-hover)] text-left text-sm ui-text-inverse transition disabled:opacity-50 disabled:shadow-none disabled:hover:ui-bg-accent ${className}`} disabled={disabled ||loading} onClick={onClick} type={type}>{loading?<LoadingSkeletonSpan width="100px" height="30px"/> :children}</button>
    )
}
