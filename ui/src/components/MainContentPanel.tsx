import {FC, PropsWithChildren} from "react"

export const MainContentPanel:FC<PropsWithChildren> = ({children}) => {
    return (
        <div className="flex flex-col grow md:ml-72 px-4 xs:px-8 transition-[margin-left]">
            {children}
        </div>
    )
}
