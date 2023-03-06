import {useEffect} from "react";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setPodcasts} from "../store/CommonSlice";
import {Card} from "../components/Card";
import {AddPodcast} from "../components/AddPodcast";
import {setModalOpen} from "../store/ModalSlice";

export const Podcasts = ()=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()

    useEffect(()=>{
        axios.get(apiURL+"/podcasts").then((response)=>{
            dispatch(setPodcasts(response.data))
        })
    },[])

    return <div className="p-5">
        <AddPodcast/>
        <div className="flex flex-1">
            <div className="flex-1"></div>
        <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
            dispatch(setModalOpen(true))
        }}></button>
        </div>
        <div className="grid grid-cols-2 xs:grid-cols-3 md:grid-cols-4 gap-2 pt-3">
            {podcasts.map((podcast, index)=>{
                return <Card podcast={podcast} key={index}/>
            })
            }
        </div>
    </div>
}
