import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSideBarCollapsed} from "../store/CommonSlice";
import {Notifications} from "./Notifications";
import {Dropdown} from "./I18nDropdown";
import {LogoutButton} from "./LogoutButton";


export const Header = ()=>{
    const dispatch = useAppDispatch()
    const sideBarCollapsed = useAppSelector(state=>state.common.sideBarCollapsed)
    const configModel = useAppSelector(state=>state.common.configModel)

    return (
        <div className="p-2 pr-4">
            <div className="flex items-center justify-between border-gray-100 md:justify-start md:space-x-10 col-span-6">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                     onClick={()=>{dispatch(setSideBarCollapsed(!sideBarCollapsed))}}
                     className=" text-white visible bg-amber-400 focus:animate-pulse p-4 h-20 hover:text-blue-500 active:scale-95 active:text-blue-600">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                </svg>
                <div className="flex flex-1"></div>
                {configModel?.oidcConfigured&&<LogoutButton/>}
                <Dropdown/>
                <Notifications/>
            </div>
        </div>
    )
}
