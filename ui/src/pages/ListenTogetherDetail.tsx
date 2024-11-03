import {useParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {configWSUrl} from "../utils/navigationUtils";
import useCommon from "../store/CommonSlice";

export const ListenTogetherDetail = ()=>{
    const id = useParams().id
    const config = useCommon(state=>state.configModel)
    const [socket, setSocket] = useState<WebSocket | null>(null)

    useEffect(() => {
        if (id == null) return
        const ws = new WebSocket(configWSUrl(config?.serverUrl!)+ "/../api/v1/publicWs/"+encodeURIComponent(id))
        ws.onopen = () => {
            ws.send("/join "+id)
        }


        setSocket(ws)

    }, [id]);


    return <div>
        <h1>Listen Together with your friends</h1>
        <div>
            <h2>Room Name</h2>
            <p>Room name: {id}</p>
            <p>Room ID: </p>
            <p>Room URL: </p>
            <p>Room Password: </p>
        </div>
    </div>
}
