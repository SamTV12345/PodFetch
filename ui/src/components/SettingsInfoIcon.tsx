import {FC} from "react";
import {useAppDispatch} from "../store/hooks";
import {setInfoHeading, setInfoModalPodcastOpen, setInfoText} from "../store/CommonSlice";

type SettingsInfoIconProps = {
    headerKey: string
    textKey: string
}

export const SettingsInfoIcon:FC<SettingsInfoIconProps> = ({textKey,headerKey})=>{
    const dispatch = useAppDispatch()
    return <button type="button"><span className="material-symbols-outlined pointer active:scale-95" onClick={()=>{
        dispatch(setInfoHeading(headerKey))
        dispatch(setInfoText(textKey))
        dispatch(setInfoModalPodcastOpen(true))
    }}>info</span></button>
}
