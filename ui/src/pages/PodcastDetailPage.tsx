import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {useEffect} from "react";
import {apiURL} from "../utils/Utilities";
import axios from "axios";
import {setSelectedEpisodes} from "../store/CommonSlice";
import {AudioPlayer} from "../components/AudioPlayer";
import {PlayIcon} from "../components/PlayIcon";

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


    return <><div className="pl-5 pt-5">
        <h1 className="text-center text-2xl">{podcast.name}</h1>
        <div className="grid place-items-center">
            <img className="w-1/2" src={podcast.image_url} alt=""/>
        </div>

        <div>
            {
                selectedEpisodes.map((episode, index)=>{
                    return <div key={index} className="grid grid-cols-[auto_1fr] gap-4">
                        <div className="flex align-baseline">
                            <PlayIcon/>
                        </div>
                        {episode.name}
                    </div>
                })
            }
        </div>
    </div>
        <AudioPlayer/>
    </>
}
