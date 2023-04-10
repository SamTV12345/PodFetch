import {useTranslation} from "react-i18next";
import {SideBarItem} from "./SideBarItem";
import {useAppSelector} from "../store/hooks";

export const SideBar  = ()=>{
    const sideBarCollapsed = useAppSelector(state=>state.common.sideBarCollapsed)
    const {t} = useTranslation()



    return <aside className={`w-full h-full float-left ${sideBarCollapsed?'hidden': 'col-span-6 md:col-span-1'} z-10 w-full bg-gray-800 flex  border-none sticky`} aria-label="Sidebar">
        <div className="py-4 px-3 bg-gray-800 h-full w-full">
            <ul className="space-y-2">
                <SideBarItem highlightPath={'./'} translationkey={t('homepage')} icon={<i className="fa-solid fa-house fa-xl"></i>}/>
                <SideBarItem highlightPath={'podcasts'} translationkey={t('podcasts')} icon={<i className="fa-solid fa-guitar   fa-xl"></i>}/>
                <SideBarItem highlightPath={"favorites"} translationkey={t('favorites')} icon={<i className="fa-solid fa-star"></i>}/>
                <SideBarItem highlightPath={"info"} translationkey={t('info')} icon={<i className="fa-solid fa-info-circle fa-xl"></i>}/>
                <SideBarItem highlightPath={"settings"} translationkey={t('settings')} icon={<i className="fa-solid fa-wrench fa-xl"/> }/>
                <SideBarItem highlightPath={"administration"} translationkey={t('administration')} icon={<i className="fa-solid fa-gavel fa-xl"/> }/>
            </ul>
        </div>
    </aside>
}
