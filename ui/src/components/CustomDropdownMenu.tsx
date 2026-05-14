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
    iconName?: string,
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
                // shadcn's DropdownMenuContent forces `w-(--anchor-width)` -
                // the popup ends up exactly as wide as the trigger (e.g. the
                // username + avatar), which clips longer item labels
                // ('System-Info', 'Administration', ...) via the also-default
                // overflow-x-hidden. !w-auto + min-w-max lets the menu size
                // itself to its widest item.
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
                                <span className="material-symbols-outlined">{menuItem.iconName}</span> {t(menuItem.translationKey)}
                            </NavLink>
                        ) : (
                            <>
                                <span className="material-symbols-outlined">{menuItem.iconName}</span> {t(menuItem.translationKey)}
                            </>
                        )}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    )
}
