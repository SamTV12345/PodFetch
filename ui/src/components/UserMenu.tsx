import {FC, useMemo} from 'react'
import { CustomDropdownMenu } from './CustomDropdownMenu'
import { MenuItem } from './CustomDropdownMenu'
import 'material-symbols/outlined.css'
import useCommon from "../store/CommonSlice";
import {$api} from "../utils/http";
import {removeLogin} from "../utils/login";


const AccountTrigger = ()=>{
    const username = useCommon(state => state.loginData)

    return <><span className="hidden md:block text-(--fg-color)">{username?.username}</span><span
        className="material-symbols-outlined text-(--fg-color) hover:text-(--fg-color-hover)">account_circle</span></>
}

export const UserMenu: FC = () => {
    const configModel = $api.useQuery('get', '/api/v1/sys/config')
    const {data, error, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        },
    })

    const menuItems: Array<MenuItem> = useMemo(()=>{
        if (isLoading || !data) {
            return []
        }
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

        if (data.role === 'admin' || !(configModel.data?.oidcConfigured && configModel.data.basicAuth)) {
            menuItems.push({
                iconName: 'settings',
                translationKey: 'settings',
                path: 'settings'
            })
        }

        if (configModel?.data?.oidcConfig || configModel?.data?.basicAuth) {
            menuItems.push({
                iconName: 'group',
                translationKey: 'administration',
                path: 'administration'
            })
        }

        if (configModel?.data?.oidcConfigured || configModel?.data?.basicAuth) {

            menuItems.push({
                iconName: 'logout',
                translationKey: 'logout',
                onClick: () => {
                    removeLogin()
                    window.location.reload()
                }
            })
        }
        return menuItems
    }, [configModel, data])


    return (
        <CustomDropdownMenu menuItems={menuItems} trigger={<AccountTrigger/>} />
    )
}
