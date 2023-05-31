import {useTranslation} from "react-i18next";
import {SideBarItem} from "./SideBarItem";
import {useAppSelector} from "../../store/hooks";
import {PodFetchLogo} from "../../icons/PodFetchLogo";
import {HeartIcon} from "../../icons/HeartIcon";
import {PodcastIcon} from "../../icons/PodcastIcon";

export const SideBar = () => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)
    const {t} = useTranslation()
    const config = useAppSelector(state => state.common.configModel)


    return (
        <aside
            className={`float-left ${sideBarCollapsed ? 'hidden' : 'col-span-6 md:col-span-1'} z-10 w-full bg-gray-950 flex  border-none sticky`}
            aria-label="Sidebar">
            <div className="py-4 px-3 bg-gray-950 w-full">
                <div className="text-amber-600 flex text-2xl gap-2"><PodFetchLogo/><h1
                    className="text-white pt-1">Podfetch</h1></div>
                <ul className="pt-20">
                    <SideBarItem highlightPath={'./'} translationkey={t('homepage')}
                                 icon={<i className="fa-solid fa-house fa-xl"></i>}/>
                    <SideBarItem highlightPath={'podcasts'} translationkey={t('all-subscriptions')}
                                 icon={<PodcastIcon/>}/>
                    <SideBarItem highlightPath={"favorites"} translationkey={t('favorites')} icon={<HeartIcon/>}/>
                    <SideBarItem highlightPath={"timeline"} translationkey={t('timeline')}
                                 icon={<i className="fa-solid fa-timeline fa-xl"/>}/>
                    {/*<SideBarItem highlightPath={"info"} translationkey={t('info')} icon={<i className="fa-solid fa-info-circle fa-xl"></i>}/>*/}
                    {<div className="display-only-mobile"><SideBarItem highlightPath={"/podcasts/search"}
                                                                       translationkey={t('search-podcasts')}
                                                                       icon={<i className="fa-solid fa-search"/>}/>
                    </div>}
                    {(config?.oidcConfig || config?.basicAuth) &&
                        <SideBarItem highlightPath={"administration"} translationkey={t('administration')}
                                     icon={<i className="fa-solid fa-gavel fa-xl"/>}/>}
                    <SideBarItem className="absolute bottom-0 w-11/12" highlightPath={"settings"}
                                 translationkey={t('settings')} icon={<i className="fa-solid fa-gear fa-xl"/>}/>
                </ul>
            </div>
        </aside>
    )
}
