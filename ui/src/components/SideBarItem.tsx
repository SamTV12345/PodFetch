import {NavLink} from "react-router-dom";
import {FC} from "react";
import {useAppDispatch} from "../store/hooks";
import {setSideBarCollapsed} from "../store/CommonSlice";

type SideBarItemProps = {
    highlightPath:string,
    translationkey: string,
    icon:React.ReactElement
}

export const SideBarItem:FC<SideBarItemProps>  =({highlightPath,translationkey,icon})=>{
    const dispatch = useAppDispatch()

    const minimizeOnMobile = ()=>{
        if(window.screen.width<768){
            dispatch(setSideBarCollapsed(true))
        }
    }
    return   <li className="sidebar" onClick={()=>minimizeOnMobile()}>
        <NavLink to={highlightPath} className="flex items-center p-2 text-base font-normal rounded-lg text-white hover:bg-gray-700 h-20">
            {icon}
            <span className="ml-3">{translationkey}</span>
        </NavLink>
    </li>
}
