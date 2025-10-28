import { type FC, useMemo } from 'react'
import { CustomDropdownMenu, type MenuItem } from './CustomDropdownMenu'
import 'material-symbols/outlined.css'
import { ADMIN_ROLE } from '../models/constants'
import useCommon from '../store/CommonSlice'
import { $api } from '../utils/http'
import { removeLogin } from '../utils/login'

const AccountTrigger = () => {
	const username = useCommon((state) => state.loginData)

	return (
		<>
			<span className="hidden md:block text-(--fg-color)">
				{username?.username}
			</span>
			<span className="material-symbols-outlined text-(--fg-color) hover:text-(--fg-color-hover)">
				account_circle
			</span>
		</>
	)
}

export const UserMenu: FC = () => {
	const configModel = $api.useQuery('get', '/api/v1/sys/config')
	const { data, error, isLoading } = $api.useQuery(
		'get',
		'/api/v1/users/{username}',
		{
			params: {
				path: {
					username: 'me',
				},
			},
		},
	)

	const menuItems: Array<MenuItem> = useMemo(() => {
		if (isLoading || !data) {
			return []
		}
		const menuItems: Array<MenuItem> = []

		if (data.role === ADMIN_ROLE) {
		}

		menuItems.push({
			iconName: 'account_circle',
			translationKey: 'profile',
			path: 'profile',
		})

		if (data.role === 'admin') {
			menuItems.push({
				iconName: 'settings',
				translationKey: 'settings',
				path: 'settings',
			})
			menuItems.push({
				iconName: 'info',
				translationKey: 'system-info',
				path: 'info',
			})
		}

		if (data.role === 'admin') {
			menuItems.push({
				iconName: 'group',
				translationKey: 'administration',
				path: 'administration',
			})
		}

		if (configModel?.data?.oidcConfigured || configModel?.data?.basicAuth) {
			menuItems.push({
				iconName: 'logout',
				translationKey: 'logout',
				onClick: () => {
					removeLogin()
					window.location.reload()
				},
			})
		}
		return menuItems
	}, [configModel, data])

	return (
		<CustomDropdownMenu menuItems={menuItems} trigger={<AccountTrigger />} />
	)
}
