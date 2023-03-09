import {useState} from "react";
import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSideBarCollapsed} from "../store/CommonSlice";
import {BellIcon} from "./BellIcon";
import {Notifications} from "./Notifications";


export const Header = ()=>{
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const sideBarCollapsed = useAppSelector(state=>state.common.sideBarCollapsed)
    const [avatarDrodownClicked, setAvatarDropdownClicked] = useState<boolean>(false)


    return (
        <div className="bg-neutral-900 w-full col-span-6 h-20 w-screen">
            <div className="flex items-center justify-between border-gray-100 py-6 md:justify-start md:space-x-10 col-span-6 w-screen h-20">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor"
                     onClick={()=>{dispatch(setSideBarCollapsed(!sideBarCollapsed))}}
                     className=" text-white  focus:animate-pulse p-4 h-20">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                </svg>
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="text-white p-4 h-20">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                </svg>

                <div className="flex flex-grow"/>
                <Notifications/>
                <div className="w-20">
                    <div className="relative inline-block text-left">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" onClick={()=>setAvatarDropdownClicked(!avatarDrodownClicked)} fill="white" className="w-16" viewBox="0 0 24 24"><path fillRule="evenodd" d="M12 2.5a5.5 5.5 0 00-3.096 10.047 9.005 9.005 0 00-5.9 8.18.75.75 0 001.5.045 7.5 7.5 0 0114.993 0 .75.75 0 101.499-.044 9.005 9.005 0 00-5.9-8.181A5.5 5.5 0 0012 2.5zM8 8a4 4 0 118 0 4 4 0 01-8 0z"></path></svg>
                        </div>
                        {avatarDrodownClicked && <div
                            className="absolute z-40 right-0 z-10 mt-2 w-56 origin-top-right rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
                            role="menu" aria-orientation="vertical" aria-labelledby="menu-button" >
                            <div className="py-1" role="none">
                            </div>
                        </div>}
                    </div>
                </div>
            </div>
        </div>
    )
}
