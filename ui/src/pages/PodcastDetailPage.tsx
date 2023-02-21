import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {useEffect} from "react";
import {apiURL} from "../utils/Utilities";
import axios from "axios";
import {setSelectedEpisodes} from "../store/CommonSlice";

export const PodcastDetailPage = () => {
    const params = useParams()
    const podcast = useAppSelector(state=>state.common.podcasts.find(podcast=>podcast.id===Number(params.id)))
    const selectedEpisodes = useAppSelector(state=>state.common.selectedEpisodes)
    const dispatch = useAppDispatch()

    useEffect(()=>{
        if (podcast){
            axios.get(apiURL+"/podcast/"+podcast.id+"/episodes")
                .then((response)=>{
                dispatch(setSelectedEpisodes(response.data))
            }
        )
    }},[podcast])

    if(podcast===undefined){
        return <div>"Nicht gefunden"</div>
    }


    return <><div className="p-5">
        <h1 className="text-center text-2xl">{podcast.name}</h1>
        <div className="grid place-items-center">
            <img className="w-1/2" src={podcast.image_url} alt=""/>
        </div>

        <div>
            {
                selectedEpisodes.map((episode, index)=>{
                    return <div key={index} className="grid grid-cols-[auto_1fr] gap-4">
                        <div className=" flex align-baseline">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth="1.5"
                                 stroke="currentColor" className="w-6 h-6 cursor-pointer">
                                <path strokeLinecap="round" strokeLinejoin="round"
                                      d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                <path strokeLinecap="round" strokeLinejoin="round"
                                      d="M15.91 11.672a.375.375 0 010 .656l-5.603 3.113a.375.375 0 01-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112z"/>
                            </svg>
                        </div>
                        {episode.name}
                    </div>
                })
            }
        </div>
    </div>
    <div className="sticky bottom-0 w-full bg-gray-800 h-10">test</div>
    </>
}
