import {FC, useEffect, useMemo} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Podcast, setPodcasts} from "../store/CommonSlice";
import {Card} from "../components/Card";
import {AddPodcast} from "../components/AddPodcast";
import {setModalOpen} from "../store/ModalSlice";
import {useLocation} from "react-router-dom";


interface PodcastsProps {
    onlyFavorites?: boolean
}

export const Podcasts:FC<PodcastsProps> = ({onlyFavorites})=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    let location = useLocation();

    useEffect(()=>{
        let url = apiURL+"/podcasts"
        if(onlyFavorites){
            url = apiURL+"/podcasts/favored"
        }
        axios.get(url)
            .then((response:AxiosResponse<Podcast[]>)=>{
                dispatch(setPodcasts(response.data))
            })
    },[location])

    return <div className="p-5">
        <AddPodcast/>
        <div className="flex flex-1">
            <div className="flex-1"></div>
        <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
            dispatch(setModalOpen(true))
        }}></button>
        </div>
        <div className="grid grid-cols-2 xs:grid-cols-3 md:grid-cols-4 gap-2 pt-3">
            {!onlyFavorites&&podcasts.map((podcast, index)=>{

                return <Card podcast={podcast} key={index}/>
            })
            }
            {
            onlyFavorites&&podcasts.filter(podcast=>podcast.favored).map((podcast)=>{
                return <Card podcast={podcast} key={podcast.id}/>
                })
            }
        </div>
    </div>
}
