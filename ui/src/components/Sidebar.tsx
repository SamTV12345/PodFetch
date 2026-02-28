import useCommon from '../store/CommonSlice'
import { useTranslation } from 'react-i18next'
import { SidebarItem } from './SidebarItem'
import 'material-symbols/outlined.css'

export const Sidebar = () => {
    const sidebarCollapsed = useCommon(state => state.sidebarCollapsed)
    const setSidebarCollapsed = useCommon(state => state.setSidebarCollapsed)
    const { t } = useTranslation()

    return (
        <div className={`fixed md:static ui-sidebar-surface h-full px-6 py-8 transition-[left] w-72 z-20 ${sidebarCollapsed ? '-left-72' : 'left-0 ui-sidebar-shadow md:shadow-none'}`} id="primary-navigation" aria-label={t('sidebar-navigation')}>

            {/* Burger menu */}
            <div className="flex items-center justify-center fixed left-0 md:-left-16 top-0 ui-bg-accent hover:ui-bg-accent-hover cursor-pointer text-white h-16 w-16 rounded-br-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_var(--color-mustard-500)] transition-all z-10"
                 onClick={()=>{setSidebarCollapsed(!sidebarCollapsed)}}>
                {sidebarCollapsed ?
                    <span className="material-symbols-outlined text-4xl!">menu</span>
                :
                    <span className="material-symbols-outlined text-4xl!">close</span>
                }
            </div>

            <span className="flex item-center gap-2 mb-10 px-4 py-3 opacity-0 md:opacity-100 transition-opacity">
                <span className="material-symbols-outlined ui-text-accent">auto_detect_voice</span>
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <ul className="flex flex-col gap-2">
                <SidebarItem iconName="home" path="./home" translationKey="homepage"/>
                <SidebarItem iconName="podcasts" path="podcasts" translationKey="all-subscriptions"/>
                <SidebarItem iconName="favorite" path="favorites" translationKey="favorites"/>
                <SidebarItem iconName="magic_button" path="timeline" translationKey="timeline"/>
                <SidebarItem iconName="query_stats" path="stats" translationKey="stats-title"/>
                <SidebarItem path="tags" translationKey="tag_other" iconName="sell"/>

                <span className="display-only-mobile">
                    <SidebarItem iconName="search" path="/podcasts/search" translationKey="search-episodes"/>
                </span>
            </ul>

        </div>
    )
}
