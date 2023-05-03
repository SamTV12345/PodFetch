import {PlayIcon} from "./PlayIcon";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime, prepareOnlinePodcastEpisode, preparePodcastEpisode, removeHTML} from "../utils/Utilities";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {CloudIcon} from "./CloudIcon";
import {InfoIcon} from "./InfoIcon";
import {
    addPodcastEpisodes,
    PodcastEpisode,
    setInfoModalPodcast,
    setInfoModalPodcastOpen}
    from "../store/CommonSlice";
import {FC} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Waypoint} from "react-waypoint";
import {useParams} from "react-router-dom";

type PodcastDetailItemProps = {
    episode: PodcastEpisode,
    index: number,
    episodesLength: number
}
export const PodcastDetailItem:FC<PodcastDetailItemProps> = ({episode, index,episodesLength}) => {
    const params = useParams()
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)
    const currentPodcastEpisode = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    const dispatch = useAppDispatch()


    if (currentPodcast === undefined) {
        return <div>"Nicht gefunden"</div>
    }


    return <>
        <div key={episode.episode_id} id={"episode_" + episode.id} className="grid grid-cols-[auto_1fr_auto] mt-2 bg-slate-900 rounded p-2 mr-2">
            <div className="grid place-items-center"><img src={currentPodcast.image_url} alt={currentPodcast.name} className="h-20 rounded"/></div>
            <div className="flex flex-col">
                <div className="ml-2 text-ago">{formatTime(episode.date_of_recording)}</div>
                <div className="ml-2 text-white font-bold mt-1">{episode.name}</div>
                <div className="line-clamp-3 text-slate-400 m-2" dangerouslySetInnerHTML={removeHTML(episode.description)}></div>
            </div>
            <div className="flex gap-5">
                <div className="grid place-items-center" key={episode.episode_id + "container"}>
                    {
                        episode.status === 'D' ?
                            <PlayIcon className="" key={episode.episode_id + "icon"}
                                      podcast={currentPodcastEpisode} onClick={() => {
                                axios.get(apiURL + "/podcast/episode/" + episode.episode_id)
                                    .then((response: AxiosResponse<PodcastWatchedModel>) => {
                                        store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(episode, response.data)))
                                        dispatch(setCurrentPodcast(currentPodcast))
                                        dispatch(setPlaying(true))
                                    })
                            }}/> : <CloudIcon className="text-2xl w-10 h-10 cursor-pointer" onClick={()=>{
                                axios.get(apiURL + "/podcast/episode/" + episode.episode_id)
                                    .then((response: AxiosResponse<PodcastWatchedModel>) => {
                                        store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode, response.data)))
                                        dispatch(setCurrentPodcast(currentPodcast))
                                        dispatch(setPlaying(true))
                                    })
                            }
                            }/>
                    }
                </div>
                <div className="grid place-items-center">
                    <InfoIcon className="mr-5 text-white " onClick={() => {
                dispatch(setInfoModalPodcast(episode))
                dispatch(setInfoModalPodcastOpen(true))
            }}/></div></div>
            {
                index===episodesLength-5&&<Waypoint key={index+"waypoint"} onEnter={()=>{
                    axios.get(apiURL+"/podcast/"+params.id+"/episodes?last_podcast_episode="+episode.date_of_recording)
                        .then((response)=>{
                            dispatch(addPodcastEpisodes(response.data))
                        })
                }
                }/>
            }
        </div>
    </>
}
