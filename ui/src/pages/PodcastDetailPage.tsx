import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {useEffect} from "react";
import {apiURL, formatTime, removeHTML} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Podcast, setSelectedEpisodes} from "../store/CommonSlice";
import {PlayIcon} from "../components/PlayIcon";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode} from "../store/AudioPlayerSlice";
import {Waypoint} from "react-waypoint";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";

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
                    return <><div key={episode.episode_id} className="grid grid-cols-[auto_1fr_3fr_auto] gap-4 mr-5">
                        <div className="flex align-baseline" key={episode.episode_id+"container"}>
                            <PlayIcon className="h-6" key={episode.episode_id+"icon"} podcast={currentPodcastEpisode} onClick={()=>{
                                axios.get(apiURL+"/podcast/episode/"+episode.episode_id)
                                    .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                                        store.dispatch(setCurrentPodcastEpisode({
                                            ...episode,
                                            time: response.data.watchedTime
                                        }))
                                        dispatch(setCurrentPodcast(currentPodcast))
                                    })
                            }}/>
                        </div>
                        <span key={episode.episode_id+"name"}>{episode.name}</span>
                        <span>{removeHTML(episode.description)}</span>
                        <span key={episode.episode_id+"date"}>{formatTime(episode.date)}</span>
                    </div>
                        <hr className="border-gray-500" key={index+"hr"}/>
                        {
                            index===selectedEpisodes.length-5&&<Waypoint key={index+"waypoint"} onEnter={()=>{
                                axios.get(apiURL+"/podcast/"+params.id+"/episodes?last_podcast_episode="+episode.date)
                                    .then((response)=>{
                                    dispatch(setSelectedEpisodes([...selectedEpisodes, ...response.data]))
                                })
                            }
                            }/>
                        }
                    </>
                })
            }
        </div>
    </div>

    </>
}
