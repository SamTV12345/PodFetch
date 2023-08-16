import { FC } from 'react'
import { NavLink } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useAppDispatch } from '../store/hooks'
import { setSidebarCollapsed } from '../store/CommonSlice'
import 'material-symbols/outlined.css'

type SidebarItemProps = {
    path: string,
    translationKey: string,
    iconName: string,
    spaceBefore?: boolean
}

export const SidebarItem: FC<SidebarItemProps> = ({ path, translationKey, iconName, spaceBefore }) => {
    const dispatch = useAppDispatch()
    const { t } = useTranslation()

    const minimizeOnMobile = () => {
        if (window.screen.width < 768) {
            dispatch(setSidebarCollapsed(true))
        }
    }

    return (
        <li onClick={() => minimizeOnMobile()} className={spaceBefore ? "space-before" : ""}>
            <NavLink className="flex items-center gap-2 px-4 py-3 rounded-lg text-sm transition-colors [&.active]:text-[--accent-color] hover:bg-[rgba(192,124,3,0.1)] [&.active]:bg-[rgba(192,124,3,0.1)]" to={path}>
                <span className="material-symbols-outlined">{iconName}</span>
                <span>{t(translationKey)}</span>
            </NavLink>
        </li>
    )
}
