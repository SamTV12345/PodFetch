import {NavLink} from "react-router-dom";
import {FC} from "react";
import {useAppDispatch} from "../store/hooks";
import {setSideBarCollapsed} from "../store/CommonSlice";

type SideBarItemProps = {
    highlightPath:string,
    translationkey: string,
    icon:React.ReactElement,
    className?:string
}

export const SideBarItem:FC<SideBarItemProps>  =({highlightPath,translationkey,icon, className})=>{
    const dispatch = useAppDispatch()

    const minimizeOnMobile = ()=>{
        if(window.screen.width<768){
            dispatch(setSideBarCollapsed(true))
        }
    }
    return   <li className={"sidebar "+className} onClick={()=>minimizeOnMobile()}>
        <NavLink to={highlightPath} className="flex pl-2  mt-1 mb-1  items-center text-base font-normal rounded-lg text-white hover:text-amber-400 hover:bg-amber-400 hover:bg-opacity-30 h-14">
            {icon}
            <span className="ml-3 hover:text-inherit">{translationkey}</span>
        </NavLink>
    </li>
}
