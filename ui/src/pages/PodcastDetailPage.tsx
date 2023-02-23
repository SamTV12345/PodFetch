import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Podcast, setSelectedEpisodes} from "../store/CommonSlice";
import {AudioPlayer} from "../components/AudioPlayer";
import {PlayIcon} from "../components/PlayIcon";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode} from "../store/AudioPlayerSlice";

export const PodcastDetailPage = () => {
    const currentPodcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const currentPodcast = useAppSelector(state=>state.audioPlayer.currentPodcast)
    const params = useParams()
    const selectedEpisodes = useAppSelector(state=>state.common.selectedEpisodes)
    const dispatch = useAppDispatch()

    useEffect(()=> {
        axios.get(apiURL + "/podcast/" + params.id).then((response: AxiosResponse<Podcast>) => {
            dispatch(setCurrentPodcast(response.data))
        }).then(() => {
            axios.get(apiURL + "/podcast/" + params.id + "/episodes")
                .then((response) => {
                    dispatch(setSelectedEpisodes(response.data))
                })
        })
    }, [])


    if(currentPodcast===undefined){
        return <div>"Nicht gefunden"</div>
    }


    return <><div className="pl-5 pt-5 overflow-y-scroll">
        <h1 className="text-center text-2xl">{currentPodcast.name}</h1>
        <div className="grid place-items-center">
            <img className="w-1/5 rounded" src={currentPodcast.image_url} alt=""/>
        </div>

        <div>
            {
                selectedEpisodes.map((episode, index)=>{
                    return <><div key={index} className="grid grid-cols-[auto_1fr] gap-4">
                        <div className="flex align-baseline">
                            <PlayIcon podcast={currentPodcastEpisode} onClick={()=>{
                                store.dispatch(setCurrentPodcastEpisode(episode))
                                dispatch(setCurrentPodcast(currentPodcast))
                            }}/>
                        </div>
                        <span>{episode.name}</span>
                    </div>
                        <hr className="border-gray-500"/>
                    </>
                })
            }
        </div>
    </div>

    </>
}
