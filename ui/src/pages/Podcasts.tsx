import {useEffect} from "react";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setPodcasts} from "../store/CommonSlice";
import {Card} from "../components/Card";

export const Podcasts = ()=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()

    useEffect(()=>{
        axios.get(apiURL+"/podcasts").then((response)=>{
            dispatch(setPodcasts(response.data))
        })
    },[])

    return <div className="p-5">
        <div className="grid grid-cols-3">
            {podcasts.map((podcast, index)=>{
                return <Card podcast={podcast} key={index}/>
            })
            }
        </div>
    </div>
}
