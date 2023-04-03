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
        <div className="bg-neutral-900 w-full col-span-6 h-20 w-screen">
            <div className="flex items-center justify-between border-gray-100 py-6 md:justify-start md:space-x-10 col-span-6 w-screen h-20">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                     onClick={()=>{dispatch(setSideBarCollapsed(!sideBarCollapsed))}}
                     className=" text-white  focus:animate-pulse p-4 h-20 hover:text-blue-500 active:scale-95 active:text-blue-600">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="text-white p-4 h-20">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                </svg>

                <div className="flex flex-grow"/>
                <Notifications/>
                {configModel?.oidcConfigured&&<LogoutButton/>}
                <Dropdown/>
                <div className="mr-2"></div>
            </div>
        </div>
    )
}
