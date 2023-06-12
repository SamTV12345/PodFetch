import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setSideBarCollapsed} from "../store/CommonSlice"
import {Dropdown} from "./I18nDropdown"
import {Notifications} from "./Notifications"
import {UserMenu} from "./UserMenu"

export const Header = ()=>{
    const dispatch = useAppDispatch()
    const sideBarCollapsed = useAppSelector(state=>state.common.sideBarCollapsed)

    return (
        <div className="flex items-center justify-between gap-8 border-gray-100 mb-8 px-8 py-6">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                    onClick={()=>{dispatch(setSideBarCollapsed(!sideBarCollapsed))}}
                    className=" text-white visible bg-amber-400 focus:animate-pulse p-4 h-20 hover:text-blue-500 active:scale-95 active:text-blue-600 md:hidden">
                <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
            </svg>

            <div className="flex-1 flex items-center justify-end gap-8 border-r border-r-stone-200 h-full pr-8">
                <Dropdown/>
                <Notifications/>
            </div>

            <UserMenu/>
        </div>
    )
}
