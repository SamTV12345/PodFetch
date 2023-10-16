import { FC } from 'react'
import { useAuth } from 'react-oidc-context'
import { CustomDropdownMenu } from './CustomDropdownMenu'
import { MenuItem } from './CustomDropdownMenu'
import 'material-symbols/outlined.css'
import useCommon from "../store/CommonSlice";

export const UserMenu: FC = () => {
    const config = useCommon(state => state.configModel)
    const configModel = useCommon(state => state.configModel)

    const menuItems: Array<MenuItem> = [
        {
            iconName: 'settings',
            translationKey: 'settings',
            path: 'settings'
        },
        {
            iconName: 'info',
            translationKey: 'system-info',
            path: 'info'
        }
    ]

    if (config?.oidcConfig || config?.basicAuth) {
        menuItems.push({
            iconName: 'group',
            translationKey: 'administration',
            path: 'administration'
        })
    }

    if (configModel?.oidcConfigured) {
        const auth = useAuth()

        menuItems.push({
            iconName: 'logout',
            translationKey: 'logout',
            onClick: () => auth.signoutRedirect()
        })
    }

    const trigger = () => (
        <span className="material-symbols-outlined text-[--fg-color] hover:text-[--fg-color-hover]">account_circle</span>
    )

    return (
        <CustomDropdownMenu menuItems={menuItems} trigger={trigger()} />
    )
}
