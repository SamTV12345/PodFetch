import {FC} from "react"
import axios, {AxiosResponse} from "axios"
import {apiURL, prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities"
import {store} from "../store/store"
import {useAppDispatch} from "../store/hooks"
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice"
import {PodcastWatchedModel} from "../models/PodcastWatchedModel"
import {TimeLineModel} from "../models/TimeLineModel"
import {selectPodcastImage} from "../pages/Homepage"
import { Podcast, PodcastEpisode } from '../store/CommonSlice'

type EpisodeCardProps = {
    podcast: Podcast,
    podcastEpisode: PodcastEpisode,
}

export const EpisodeCard:FC<EpisodeCardProps> = ({podcast, podcastEpisode}) => {
    const dispatch = useAppDispatch()

    return (
        <div className="group cursor-pointer" key={podcastEpisode.episode_id+"dv"} onClick={()=>{
            axios.get(apiURL+"/podcast/episode/"+podcastEpisode.episode_id)
                .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                    if (podcastEpisode.local_image_url.trim().length>1){
                        store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(podcastEpisode, response.data)))
                    }
                    else{
                        store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(podcastEpisode, response.data)))
                    }
                    dispatch(setCurrentPodcast(podcast))
                    dispatch(setPlaying(true))
                })
        }}>

            {/* Thumbnail */}
            <div className="relative aspect-square bg-center bg-cover mb-2 overflow-hidden rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)] w-full" key={podcastEpisode.episode_id} style={{backgroundImage: `url("${selectPodcastImage(podcastEpisode)}")`}}>
                <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="material-symbols-outlined !text-7xl text-white group-active:scale-90" key={podcastEpisode.episode_id+"icon"}>play_circle</span>
                </div>
            </div>

            {/* Titles */}
            <div>
                <span className="block font-bold leading-[1.2] mb-2 text-sm text-stone-900 transition-color group-hover:text-stone-600">{podcastEpisode.name}</span>
                <span className="block leading-[1.2] text-xs text-stone-900">{podcast.name}</span>
            </div>
        </div>
    )
}
