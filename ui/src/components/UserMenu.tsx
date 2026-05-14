import {FC, useMemo} from 'react'
import { CustomDropdownMenu, MenuItem } from './CustomDropdownMenu'
import { CircleUserRound, Info, LogOut, Settings, Users } from 'lucide-react'
import useCommon from "../store/CommonSlice";
import {$api} from "../utils/http";
import {removeLogin} from "../utils/login";
import {ADMIN_ROLE} from "../models/constants";


const AccountTrigger = ()=>{
    const username = useCommon(state => state.loginData)

    return <>
        <span className="hidden md:block ui-text">{username?.username}</span>
        <CircleUserRound className="ui-text hover:ui-text-hover" size={20} />
    </>
}

export const UserMenu: FC = () => {
    const configModel = $api.useQuery('get', '/api/v1/sys/config')
    const {data, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
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

        ]

        if (data.role === ADMIN_ROLE) {

        }

        menuItems.push({
            icon: <CircleUserRound size={16} />,
            translationKey: 'profile',
            path: 'profile'
        })

        if (data.role === 'admin') {
            menuItems.push({
                icon: <Settings size={16} />,
                translationKey: 'settings',
                path: 'settings'
            })
            menuItems.push({
                icon: <Info size={16} />,
                translationKey: 'system-info',
                path: 'info'
            })
        }

        if (data.role === 'admin') {
            menuItems.push({
                icon: <Users size={16} />,
                translationKey: 'administration',
                path: 'administration'
            })
        }

        if (configModel?.data?.oidcConfigured || configModel?.data?.basicAuth) {
            menuItems.push({
                icon: <LogOut size={16} />,
                translationKey: 'logout',
                onClick: () => {
                    removeLogin()
                    window.location.reload()
                }
            })
        }
        return menuItems
    }, [configModel, data, isLoading])


    return (
        <CustomDropdownMenu menuItems={menuItems} trigger={<AccountTrigger/>} />
    )
}
