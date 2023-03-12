import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {useEffect} from "react";
import {apiURL, formatTime} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Podcast, setInfoModalPodcast, setInfoModalPodcastOpen, setSelectedEpisodes} from "../store/CommonSlice";
import {PlayIcon} from "../components/PlayIcon";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {Waypoint} from "react-waypoint";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {CloudIcon} from "../components/CloudIcon";
import {InfoIcon} from "../components/InfoIcon";
import {PodcastInfoModal} from "../components/PodcastInfoModal";

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
                    if(params.podcastid){
                        const element = document.getElementById("episode_"+params.podcastid)
                        if(element){
                            element.scrollIntoView({behavior: "smooth", block: "start", inline: "nearest"})
                        }
                    }
                })
        })
    }, [])

    useEffect(()=>{

            if(params.podcastid){
                const element = document.getElementById("episode_"+params.podcastid)
                if(element){
                    element.scrollIntoView({behavior: "smooth", block: "start", inline: "nearest"})
                }
            }
        },

        [params])


    if(currentPodcast===undefined){
        return <div>"Nicht gefunden"</div>
    }

    return <><div className="pl-5 pt-5 overflow-y-scroll">
        <PodcastInfoModal/>
        <h1 className="text-center text-2xl">{currentPodcast.name}
            <i className="fa-solid fa-arrows-rotate hover:text-slate-400 active:text-slate-800 active:scale-95 ml-1" onClick={()=>{
                axios.post(apiURL+"/podcast/"+params.id+"/refresh")
                    .then(()=>{
                       console.log("Refreshed")
                    })
            }}></i></h1>
        <div className="grid place-items-center">
            <img className="w-1/5 rounded" src={currentPodcast.image_url} alt=""/>
        </div>

        <div>
            {
                selectedEpisodes.map((episode, index)=>{
                    return <><div key={episode.episode_id} id={"episode_"+episode.id} className="grid grid-cols-[auto_7fr_auto_1fr] gap-4 mr-5">
                        <div className="grid place-items-center" key={episode.episode_id+"container"}>
                            {
                                episode.status==='D'?
                                <PlayIcon className="h-6" key={episode.episode_id+"icon"} podcast={currentPodcastEpisode} onClick={()=>{
                                    axios.get(apiURL+"/podcast/episode/"+episode.episode_id)
                                        .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                                            store.dispatch(setCurrentPodcastEpisode({
                                                ...episode,
                                                time: response.data.watchedTime
                                            }))
                                            dispatch(setCurrentPodcast(currentPodcast))
                                            dispatch(setPlaying(true))
                                        })
                                }}/>:<CloudIcon/>
                            }
                        </div>
                        <span key={episode.episode_id+"name"}>{episode.name}</span>
                        <span><InfoIcon onClick={()=>{
                            console.log("Clicked")
                            dispatch(setInfoModalPodcast(episode))
                            dispatch(setInfoModalPodcastOpen(true))
                        }}/></span>
                        <span key={episode.episode_id+"date"} className="flex gap-5">
                            {formatTime(episode.date_of_recording)}
                        </span>
                    </div>
                        <hr className="border-gray-500" key={index+"hr"}/>
                        {
                            index===selectedEpisodes.length-5&&<Waypoint key={index+"waypoint"} onEnter={()=>{
                                axios.get(apiURL+"/podcast/"+params.id+"/episodes?last_podcast_episode="+episode.date_of_recording)
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
