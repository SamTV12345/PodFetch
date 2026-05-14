import {FC} from "react";
import {Info} from "lucide-react";

type InfoIconProps = {
    className?: string
    onClick?: () => void
}
export const InfoIcon:FC<InfoIconProps> = ({onClick, className}) => {
    return <Info size={32} className={`ui-icon hover:ui-icon-hover active:scale-95 cursor-pointer ${className ?? ''}`} onClick={onClick}/>
}
