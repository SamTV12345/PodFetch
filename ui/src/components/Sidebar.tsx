import useCommon from '../store/CommonSlice'
import { useTranslation } from 'react-i18next'
import { SidebarItem } from './SidebarItem'
import { BarChart2, Compass, Heart, Home, Menu as MenuIcon, Mic, Podcast, Search, Sparkles, Tag, X } from 'lucide-react'

const ICON_SIZE = 18

export const Sidebar = () => {
    const sidebarCollapsed = useCommon(state => state.sidebarCollapsed)
    const setSidebarCollapsed = useCommon(state => state.setSidebarCollapsed)
    const { t } = useTranslation()

    return (
        <div className={`fixed md:static ui-sidebar-surface h-full px-6 py-8 transition-[left] w-72 z-20 ${sidebarCollapsed ? '-left-72' : 'left-0 ui-sidebar-shadow md:shadow-none'}`} id="primary-navigation" aria-label={t('sidebar-navigation')}>

            {/* Burger menu */}
            <div className="flex items-center justify-center fixed left-0 md:-left-16 top-0 ui-bg-accent hover:ui-bg-accent-hover cursor-pointer text-white h-16 w-16 rounded-br-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_var(--color-mustard-500)] transition-all z-10"
                 onClick={()=>{setSidebarCollapsed(!sidebarCollapsed)}}>
                {sidebarCollapsed ? <MenuIcon size={32} /> : <X size={32} />}
            </div>

            <span className="flex item-center gap-2 mb-10 px-4 py-3 opacity-0 md:opacity-100 transition-opacity">
                <Mic className="ui-text-accent" size={22} />
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <ul className="flex flex-col gap-2">
                <SidebarItem icon={<Home size={ICON_SIZE} />} path="./home" translationKey="homepage"/>
                <SidebarItem icon={<Podcast size={ICON_SIZE} />} path="podcasts" translationKey="all-subscriptions"/>
                <SidebarItem icon={<Compass size={ICON_SIZE} />} path="discover" translationKey="discover"/>
                <SidebarItem icon={<Heart size={ICON_SIZE} />} path="favorites" translationKey="favorites"/>
                <SidebarItem icon={<Sparkles size={ICON_SIZE} />} path="timeline" translationKey="timeline"/>
                <SidebarItem icon={<BarChart2 size={ICON_SIZE} />} path="stats" translationKey="stats-title"/>
                <SidebarItem icon={<Tag size={ICON_SIZE} />} path="tags" translationKey="tag_other"/>

                <span className="display-only-mobile">
                    <SidebarItem icon={<Search size={ICON_SIZE} />} path="/podcasts/search" translationKey="search-episodes"/>
                </span>
            </ul>

        </div>
    )
}
