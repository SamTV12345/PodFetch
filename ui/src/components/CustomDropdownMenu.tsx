import { FC, ReactNode } from 'react'
import { NavLink } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import * as DropdownMenu from '@radix-ui/react-dropdown-menu'

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
        <DropdownMenu.Root>
            <DropdownMenu.Trigger className="flex items-center">
                {trigger}
            </DropdownMenu.Trigger>

            <DropdownMenu.Portal>
                <DropdownMenu.Content className="bg-(--bg-color) py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-30">

                    {menuItems.map((menuItem) => (
                        <DropdownMenu.Item key={menuItem.translationKey}>
                            {menuItem.onClick ?
                                <span className="flex items-center gap-2 cursor-pointer px-6 py-2 text-sm text-(--fg-color) hover:text-(--fg-color-hover)" onClick={menuItem.onClick}>
                                    <span className="material-symbols-outlined">{menuItem.iconName}</span> {t(menuItem.translationKey)}
                                </span>
                            : menuItem.path ?
                                <NavLink className="flex items-center gap-2 cursor-pointer px-6 py-2 text-sm text-(--fg-color) hover:text-(--fg-color-hover)" to={menuItem.path}>
                                    <span className="material-symbols-outlined">{menuItem.iconName}</span> {t(menuItem.translationKey)}
                                </NavLink>
                            :
                                <span className="flex items-center gap-2 cursor-pointer px-6 py-2 text-sm text-(--fg-color) hover:text-(--fg-color-hover)">
                                    <span className="material-symbols-outlined">{menuItem.iconName}</span> {t(menuItem.translationKey)}
                                </span>
                            }
                        </DropdownMenu.Item>
                    ))}

                    <DropdownMenu.Arrow className="fill-(--bg-color)" />
                </DropdownMenu.Content>
            </DropdownMenu.Portal>
        </DropdownMenu.Root>
    )
}
