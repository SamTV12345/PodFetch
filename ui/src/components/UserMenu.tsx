import {FC, useMemo} from 'react'
import { useAuth } from 'react-oidc-context'
import { CustomDropdownMenu } from './CustomDropdownMenu'
import { MenuItem } from './CustomDropdownMenu'
import 'material-symbols/outlined.css'
import useCommon from "../store/CommonSlice";


const AccountTrigger = ()=>{
    const username = useCommon(state => state.loginData)

    return <button className="flex gap-3"><span className="hidden md:block text-(--fg-color)">{username?.username}</span><span className="material-symbols-outlined text-(--fg-color) hover:text-(--fg-color-hover)">account_circle</span></button>
}

export const UserMenu: FC = () => {
    const config = useCommon(state => state.configModel)
    const configModel = useCommon(state => state.configModel)
    const loggedInUser = useCommon(state => state.loggedInUser)

    const menuItems: Array<MenuItem> = useMemo(()=>{
        const menuItems: Array<MenuItem> = [
            {
                iconName: 'info',
                translationKey: 'system-info',
                path: 'info'
            }
        ]

        menuItems.push({
            iconName: 'account_circle',
            translationKey: 'profile',
            path: 'profile'
        })

        if (loggedInUser?.role === 'admin' || !(config?.oidcConfigured && config.basicAuth)) {
            menuItems.push({
                iconName: 'settings',
                translationKey: 'settings',
                path: 'settings'
            })
        }

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

        if(configModel?.basicAuth){
            menuItems.push({
                iconName: 'logout',
                translationKey: 'logout',
                onClick: () => {
                    localStorage.removeItem('auth')
                    sessionStorage.removeItem('auth')
                    window.location.reload()
                }
            })
        }
        return menuItems
    }, [configModel,config, loggedInUser])


    return (
        <CustomDropdownMenu menuItems={menuItems} trigger={<AccountTrigger/>} />
    )
}
