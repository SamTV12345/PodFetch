import useCommon from '../store/CommonSlice'
import { LanguageDropdown } from './I18nDropdown'
import { Notifications } from './Notifications'
import { ThemeSelector } from './ThemeSelector'
import { UserMenu } from './UserMenu'

export const Header = () => {
	return (
		<div className="flex items-center justify-end gap-8 mb-8 py-6">
			<LanguageDropdown />
			<ThemeSelector />
			<Notifications />
			<div className="hidden xs:block border-r border-r-(--border-color) h-full w-1"></div>
			<UserMenu />
		</div>
	)
}
