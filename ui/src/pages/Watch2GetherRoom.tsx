import {useEffect} from "react";
import {configWSUrl} from "../utils/navigationUtils";
import useCommon from "../store/CommonSlice";

export const Watch2GetherRoom = ()=>{
    const configModel = useCommon(state => state.configModel)
    const watchTogetherSocket = new WebSocket(configWSUrl(configModel?.serverUrl!)+ "")


    useEffect(() => {

    }, []);
}
