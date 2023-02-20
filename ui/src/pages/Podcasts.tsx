import {useEffect} from "react";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setPodcasts} from "../store/CommonSlice";

export const Podcasts = ()=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()

    useEffect(()=>{
        axios.get(apiURL+"/podcasts").then((response)=>{
            dispatch(setPodcasts(response.data))
        })
    },[])
    return <div>
        {podcasts.map((podcast, index)=>{
            return <div>{podcast.name}</div>
        })
        }
    </div>
}
