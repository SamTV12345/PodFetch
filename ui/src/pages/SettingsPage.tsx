import { useTranslation } from 'react-i18next'
import { NavLink, Outlet } from 'react-router-dom'
import { ConfirmModal } from '../components/ConfirmModal'
import { Heading1 } from '../components/Heading1'
import { InfoModal } from '../components/InfoModal'

export const SettingsPage = () => {
	const { t } = useTranslation()

	return (
		<div>
			<ConfirmModal />

			<Heading1 className="mb-10">{t('settings')}</Heading1>

			{/* Tabs */}
			<div
				className={`
                scrollbox-x mb-10 py-2
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}
			>
				<ul className="flex gap-2 border-b border-(--border-color) min-w-fit text-(--fg-secondary-color) settings-selector ">
					<li className={`cursor-pointer inline-block px-2 py-4`}>
						<NavLink to="retention" className="">
							{t('data-retention')}
						</NavLink>
					</li>
					<li className={`cursor-pointer inline-block px-2 py-4`}>
						<NavLink to="opml">{t('opml-export')}</NavLink>
					</li>
					<li className={`cursor-pointer inline-block px-2 py-4`}>
						<NavLink to="naming">{t('podcast-naming')}</NavLink>
					</li>
					<li className={`cursor-pointer inline-block px-2 py-4`}>
						<NavLink to="podcasts">{t('manage-podcasts')}</NavLink>
					</li>
					<li className={`cursor-pointer inline-block px-2 py-4`}>
						<NavLink to="gpodder">{t('manage-gpodder-podcasts')}</NavLink>
					</li>
				</ul>
			</div>

			<div className="max-w-(--breakpoint-md)">
				<Outlet />
			</div>

			<InfoModal />
		</div>
	)
}
