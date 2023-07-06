import {FC} from "react"
import {useAuth} from "react-oidc-context"
import {useAppSelector} from "../store/hooks"
import {CustomDropdownMenu} from "./CustomDropdownMenu"
import {MenuItem} from "./CustomDropdownMenu"
import "material-symbols/outlined.css"

export const UserMenu:FC = () => {
    
    const config = useAppSelector(state => state.common.configModel)
    const configModel = useAppSelector(state=>state.common.configModel)

    const menuItems:Array<MenuItem> = [
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

    const trigger = () => <span className="material-symbols-outlined text-stone-900 hover:text-stone-600">account_circle</span>

    return <CustomDropdownMenu menuItems={menuItems} trigger={trigger()}/>
}
