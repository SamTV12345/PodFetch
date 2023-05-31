import {NavLink} from "react-router-dom";
import {FC} from "react";
import {useAppDispatch} from "../../store/hooks";
import {setSideBarCollapsed} from "../../store/CommonSlice";

type SideBarItemProps = {
    highlightPath: string,
    translationKey: string,
    icon: React.ReactElement,
    spaceBefore?: boolean
}

export const SideBarItem: FC<SideBarItemProps> = ({highlightPath, translationKey, icon, spaceBefore}) => {
    const dispatch = useAppDispatch()

    const minimizeOnMobile = () => {
        if (window.screen.width < 768) {
            dispatch(setSideBarCollapsed(true))
        }
    }
    return (
        <li onClick={() => minimizeOnMobile()} className={spaceBefore ? "space-before" : ""}>
            <NavLink to={highlightPath}>
                {icon}
                <span>{translationKey}</span>
            </NavLink>
        </li>
    )
}
