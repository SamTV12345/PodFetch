import {FC} from "react"
import {Waypoint} from "react-waypoint"
import axios, {AxiosResponse} from "axios"
import {apiURL, prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities"
import {store} from "../store/store"
import {useAppDispatch} from "../store/hooks"
import {addTimelineEpisodes} from "../store/CommonSlice"
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice"
import {PodcastWatchedModel} from "../models/PodcastWatchedModel"
import {TimelineHATEOASModel, TimeLineModel} from "../models/TimeLineModel"
import {selectPodcastImage} from "../pages/Homepage"

type TimelineEpisodeCardProps = {
    podcastEpisode: TimeLineModel,
    index: number,
    timelineLength: number,
    totalLength: number,
    timeLineEpisodes: TimelineHATEOASModel
}

export const TimelineEpisodeCard:FC<TimelineEpisodeCardProps> = ({podcastEpisode, index, timelineLength, timeLineEpisodes}) => {
    const dispatch = useAppDispatch()

    return <>
        <div className="group cursor-pointer" key={podcastEpisode.podcast_episode.episode_id+"dv"} onClick={()=>{
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
        }}>

            {/* Thumbnail */}
            <div className="relative aspect-square bg-center bg-cover mb-2 overflow-hidden rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)] w-full" key={podcastEpisode.podcast_episode.episode_id} style={{backgroundImage: `url("${selectPodcastImage(podcastEpisode.podcast_episode)}")`}}>
                <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="material-symbols-outlined !text-7xl text-white group-active:scale-90" key={podcastEpisode.podcast_episode.episode_id+"icon"}>play_circle</span>
                </div>
            </div>

            {/* Titles */}
            <div>
                <span className="block font-bold leading-[1.2] mb-2 text-sm text-stone-900 transition-color group-hover:text-stone-600">{podcastEpisode.podcast_episode.name}</span>
                <span className="block leading-[1.2] text-xs text-stone-900">{podcastEpisode.podcast.name}</span>
            </div>
        </div>

        {/*Infinite scroll */
        timeLineEpisodes.data.length === index+1 &&
            <Waypoint key={index+"waypoint"} onEnter={()=>{
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
    </>
}
