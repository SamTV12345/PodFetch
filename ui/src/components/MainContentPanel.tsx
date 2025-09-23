import { FC, PropsWithChildren } from 'react'
import useCommon from '../store/CommonSlice'

export const MainContentPanel: FC<PropsWithChildren> = ({ children }) => {
    const setSidebarCollapsed = useCommon(state => state.setSidebarCollapsed)
    const sidebarCollapsed = useCommon(state => state.sidebarCollapsed)

    return (
        <div className="flex flex-col px-4 xs:px-8 overflow-y-auto pb-28">
            {/* Scrim for sidebar */}
            <div className={`fixed inset-0 z-10 ${sidebarCollapsed ? 'hidden' : 'block md:hidden'}`} onClick={() => { setSidebarCollapsed(!sidebarCollapsed) }}></div>

            {children}
        </div>
    )
}
