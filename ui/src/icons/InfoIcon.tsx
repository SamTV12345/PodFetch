import {FC} from "react";

type InfoIconProps = {
    className?: string
    onClick?: () => void
}
export const InfoIcon:FC<InfoIconProps> = ({onClick, className}) => {
    return <i className={`fa-solid fa-circle-info fa-2x ui-icon hover:ui-icon-hover active:scale-95 ${className}`} onClick={()=> onClick ? onClick() :''}/>
}
