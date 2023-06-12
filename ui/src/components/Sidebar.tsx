import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSidebarCollapsed} from "../store/CommonSlice"
import {SidebarItem} from "./SidebarItem";
import "material-symbols/outlined.css"

export const Sidebar = () => {
    const dispatch = useAppDispatch()
    const sidebarCollapsed = useAppSelector(state => state.common.sidebarCollapsed)
    const {t} = useTranslation()

    return (
        <div className={`fixed bg-stone-950 px-6 py-8 text-white transition-[left] w-72 z-20 ${sidebarCollapsed ? '-left-72 md:left-0' : '!left-0 shadow-[4px_0_16px_rgba(0,0,0,0.5)] md:shadow-none'}`} id="primary-navigation" aria-label="Sidebar">

            {/* Burger menu */}
            <div className="flex items-center justify-center fixed left-0 md:-left-16 top-0 bg-mustard-600 hover:bg-mustard-500 cursor-pointer text-white h-16 w-16 rounded-br-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_16px_theme(colors.mustard.500)] transition-all z-30 " onClick={()=>{dispatch(setSidebarCollapsed(!sidebarCollapsed))}}>
                {sidebarCollapsed ?
                    <span className="material-symbols-outlined !text-4xl">menu</span>
                :
                    <span className="material-symbols-outlined !text-4xl">close</span>
                }
            </div>

            <span className="flex item-center gap-2 mb-10 px-4 py-3 opacity-0 md:opacity-100 transition-opacity">
                <span className="material-symbols-outlined text-mustard-600">auto_detect_voice</span>
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <ul className="flex flex-col gap-2">
                <SidebarItem iconName="home" path="./" translationKey="homepage"/>
                <SidebarItem iconName="podcasts" path="podcasts" translationKey="all-subscriptions"/>
                <SidebarItem iconName="favorite" path="favorites" translationKey="favorites"/>
                <SidebarItem iconName="magic_button" path="timeline" translationKey="timeline"/>

                <div className="display-only-mobile">
                    <SidebarItem iconName="search" path="/podcasts/search" translationKey="search-podcasts"/>
                </div>
            </ul>

        </div>
    )
}
