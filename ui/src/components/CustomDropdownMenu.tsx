import { FC, ReactNode } from 'react'
import { NavLink } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

type CustomDropdownMenuProps = {
    menuItems: Array<MenuItem>,
    trigger: ReactNode
}

export type MenuItem = {
    icon?: ReactNode,
    translationKey: string,
    onClick?: () => void,
    path?: string
}

export const CustomDropdownMenu: FC<CustomDropdownMenuProps> = ({ menuItems, trigger }) => {
    const { t } = useTranslation()

    return (
        <DropdownMenu>
            <DropdownMenuTrigger className="flex items-center">
                {trigger}
            </DropdownMenuTrigger>

            <DropdownMenuContent
                align="end"
                className="!w-auto min-w-max py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]"
            >
                {menuItems.map((menuItem) => (
                    <DropdownMenuItem
                        key={menuItem.translationKey}
                        onClick={menuItem.onClick}
                        className="flex items-center gap-2 cursor-pointer px-6 py-2 text-sm ui-text hover:ui-text-hover"
                    >
                        {menuItem.path ? (
                            <NavLink className="flex items-center gap-2 w-full" to={menuItem.path}>
                                {menuItem.icon}<span>{t(menuItem.translationKey)}</span>
                            </NavLink>
                        ) : (
                            <>
                                {menuItem.icon}<span>{t(menuItem.translationKey)}</span>
                            </>
                        )}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    )
}
