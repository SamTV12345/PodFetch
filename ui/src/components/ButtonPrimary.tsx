import {FC, ReactNode} from "react"

type ButtonPrimaryProps = {
    children: ReactNode,
    className?: string,
    onClick: () => void
}

export const ButtonPrimary:FC<ButtonPrimaryProps> = ({children, className, onClick}) => {
    return <button className={"bg-mustard-600 hover:bg-mustard-500 px-3 py-2 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_theme(colors.mustard.500)] text-sm text-white transition-colors " + className} onClick={onClick}>{children}</button>
}
