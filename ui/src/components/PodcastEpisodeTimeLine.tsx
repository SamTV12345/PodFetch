import {PlayIcon} from "../icons/PlayIcon";
import axios, {AxiosResponse} from "axios";
import {apiURL, prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {FC} from "react";
import {selectPodcastImage} from "../pages/Homepage";
import {TimelineHATEOASModel, TimeLineModel} from "../models/TimeLineModel";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Waypoint} from "react-waypoint";
import {addPodcastEpisodes, addTimelineEpisodes} from "../store/CommonSlice";


type PodcastEpisodeTimeLineProps = {
    podcastEpisode: TimeLineModel,
    index: number,
    timelineLength: number,
    totalLength: number,
    timeLineEpisodes: TimelineHATEOASModel
}

export const PodcastEpisodeTimeLine:FC<PodcastEpisodeTimeLineProps> = ({podcastEpisode, index, timelineLength, timeLineEpisodes}) => {
    const dispatch = useAppDispatch()

    return <div key={podcastEpisode.podcast_episode.episode_id+"dv"}
                className="max-w-sm rounded-lg shadow bg-slate-700 border-gray-700">
        <div className="relative" key={podcastEpisode.podcast_episode.episode_id}>
            <img src={selectPodcastImage(podcastEpisode.podcast_episode)} alt="" className=""/>
            <div className="absolute left-0 top-0 w-full h-full hover:bg-gray-500 opacity-80 z-10 grid place-items-center play-button-background">
                <PlayIcon key={podcastEpisode.podcast_episode.episode_id+"icon"} podcast={podcastEpisode.podcast_episode} className="w-20 h-20 opacity-0" onClick={()=>{
                    axios.get(apiURL+"/podcast/episode/"+podcastEpisode.podcast_episode.episode_id)
                        .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                            if (podcastEpisode.podcast_episode.local_image_url.trim().length>1){
                                store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(podcastEpisode.podcast_episode, response.data)))
                            }
                            else{
                                store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(podcastEpisode.podcast_episode, response.data)))
                            }
                            dispatch(setCurrentPodcast(podcastEpisode.podcast))
                            dispatch(setPlaying(true))
                        })
                }}/>
            </div>
            {
                timeLineEpisodes.data.length === index+1 &&<Waypoint key={index+"waypoint"} onEnter={()=>{
                    axios.get(apiURL+"/podcasts/timeline", {
                        params:{
                            lastTimestamp: podcastEpisode.podcast_episode.date_of_recording,
                            favoredOnly: store.getState().common.filters?.onlyFavored
                        }
                    })
                        .then((response:AxiosResponse<TimelineHATEOASModel>)=>{
                            dispatch(addTimelineEpisodes(response.data))
                        })
                }
                }/>
            }
        </div>
        <div className="p-5">
            <h5 className="mb-2 text-2xl font-bold tracking-tight text-white break-words">{podcastEpisode.podcast_episode.name}</h5>
        </div>
    </div>
}
