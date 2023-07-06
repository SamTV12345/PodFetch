import {FC, PropsWithChildren} from "react"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setSidebarCollapsed} from "../store/CommonSlice"

export const MainContentPanel:FC<PropsWithChildren> = ({children}) => {
    const dispatch = useAppDispatch()
    const sidebarCollapsed = useAppSelector(state => state.common.sidebarCollapsed)

    return (
        <div className="flex flex-col px-4 xs:px-8">
            {/* Scrim for sidebar */}
            <div className={`fixed inset-0 z-10 ${sidebarCollapsed ? 'hidden' : 'block md:hidden'}`} onClick={() => {dispatch(setSidebarCollapsed(!sidebarCollapsed))}}></div>

            {children}
        </div>
    )
}
