import {FC, PropsWithChildren} from "react";
import {useAppSelector} from "../../store/hooks";
import "./style.scss"

const ContentPanel: FC<PropsWithChildren> = ({children}) => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)

    return (
        <div className={`content-panel ${sideBarCollapsed ? "closed" : "open"}`}>
            {children}
        </div>
    )
}

export default ContentPanel