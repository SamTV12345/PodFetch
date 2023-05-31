import {useTranslation} from "react-i18next";
import {SideBarItem} from "./SideBarItem";
import {useAppSelector} from "../../store/hooks";
import {PodFetchIcon} from "../../icons/PodFetchIcon";
import "./style.scss"
import {MdFavoriteBorder, MdOutlineHome, MdOutlineSettings, MdPodcasts, MdTimeline} from "react-icons/md";

export const SideBar = () => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)
    const {t} = useTranslation()
    const config = useAppSelector(state => state.common.configModel)


    return (
        <aside className={`side-bar ${sideBarCollapsed ? 'closed' : 'open'}`} aria-label="Sidebar">
            <div className="logo-section">
                <PodFetchIcon/>
                <h1 className="text-white pt-1">Podfetch</h1>
            </div>

            <ul className="navigation-section">
                <SideBarItem highlightPath={'./'} translationKey={t('homepage')}
                             icon={<MdOutlineHome />}/>
                <SideBarItem highlightPath={'podcasts'} translationKey={t('all-subscriptions')}
                             icon={<MdPodcasts/>}/>
                <SideBarItem highlightPath={"favorites"} translationKey={t('favorites')} icon={<MdFavoriteBorder/>}/>
                <SideBarItem highlightPath={"timeline"} translationKey={t('timeline')}
                             icon={<MdTimeline/>}/>
                {/*<SideBarItem highlightPath={"info"} translationkey={t('info')} icon={<i className="fa-solid fa-info-circle fa-xl"></i>}/>*/}
                {<div className="display-only-mobile"><SideBarItem highlightPath={"/podcasts/search"}
                                                                   translationKey={t('search-podcasts')}
                                                                   icon={<i className="fa-solid fa-search"/>}/>
                </div>}
                {(config?.oidcConfig || config?.basicAuth) &&
                    <SideBarItem highlightPath={"administration"} translationKey={t('administration')}
                                 icon={<i className="fa-solid fa-gavel fa-xl"/>}/>}
                <SideBarItem highlightPath={"settings"} spaceBefore
                             translationKey={t('settings')} icon={<MdOutlineSettings/>}/>
            </ul>
        </aside>
    )
}
