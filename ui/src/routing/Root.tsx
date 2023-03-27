import {FC} from "react";
import {useAppSelector} from "../store/hooks";
import {Header} from "../components/Header";
import {SideBar} from "../components/SideBar";
import {Outlet} from "react-router-dom";
import {AudioComponents} from "../components/AudioComponents";
import {Search} from "../components/Search";

export const Root = () => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)

    return <>
        <div className="grid  grid-rows-[auto_1fr] h-full md:grid-cols-[300px_1fr]">
            <Header/>
            <SideBar/>
            <div
                className={`col-span-6 md:col-span-5 ${sideBarCollapsed ? 'xs:col-span-5' : 'hidden'} md:block w-full overflow-x-auto`}>
                <div className="grid grid-rows-[1fr_auto] h-full ">
                    <Outlet/>
                    <AudioComponents/>
                </div>
            </div>
        </div>
        <Search/></>
}
