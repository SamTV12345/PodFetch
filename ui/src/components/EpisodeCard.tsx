import { FC} from 'react'
import axios, { AxiosResponse } from 'axios'
import { apiURL, prepareOnlinePodcastEpisode, preparePath, preparePodcastEpisode } from '../utils/Utilities'
import { store } from '../store/store'
import { useAppDispatch } from '../store/hooks'
import { setCurrentPodcast, setCurrentPodcastEpisode, setPlaying } from '../store/AudioPlayerSlice'
import { PodcastWatchedModel } from '../models/PodcastWatchedModel'
import {Podcast, PodcastEpisode, setPodcastAlreadyPlayed, setPodcastEpisodeAlreadyPlayed} from '../store/CommonSlice'
import { PodcastWatchedEpisodeModel } from '../models/PodcastWatchedEpisodeModel'

type EpisodeCardProps = {
    podcast: Podcast,
    podcastEpisode: PodcastEpisode,
    totalTime?: number,
    watchedTime?: number
}

const isPodcastWatchedEpisodeModel = (podcast: PodcastWatchedEpisodeModel|PodcastEpisode): podcast is PodcastWatchedEpisodeModel => {
    return (podcast as PodcastWatchedEpisodeModel).watchedTime !== undefined;
}

export const selectPodcastImage = (podcast: PodcastWatchedEpisodeModel|PodcastEpisode) => {
    if (isPodcastWatchedEpisodeModel(podcast)){
        if(podcast.podcastEpisode.local_image_url.length>1){
            return preparePath(podcast.podcastEpisode.local_image_url)
        }
        else{
            return podcast.podcastEpisode.image_url
        }
    }
    else{
        if(podcast.local_image_url.trim().length>1){
            return preparePath(podcast.local_image_url)
        }
        else{
            return podcast.image_url
        }
    }

}

export const EpisodeCard: FC<EpisodeCardProps> = ({ podcast, podcastEpisode, totalTime, watchedTime }) => {
    const dispatch = useAppDispatch()

    return (
        <div className="group cursor-pointer" key={podcastEpisode.episode_id+"dv"} onClick={()=>{
            axios.get(apiURL + '/podcast/episode/' + podcastEpisode.episode_id)
                .then((response: AxiosResponse<PodcastWatchedModel>) => {

                    const playedPercentage = response.data.watchedTime * 100 / podcastEpisode.total_time
                    if(playedPercentage < 95) {
                        podcastEpisode.status === 'D'
                            ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(podcastEpisode, response.data)))
                            : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(podcastEpisode, response.data)))

                        dispatch(setCurrentPodcast(podcast))
                        dispatch(setPlaying(true))
                    }
                    else{
                        dispatch(setPodcastEpisodeAlreadyPlayed({
                            podcastEpisode:{
                                podcastEpisode: podcastEpisode
                            },
                            podcastWatchModel: response.data
                        }))
                        dispatch(setPodcastAlreadyPlayed(true))
                    }
                })
        }}>

            {/* Thumbnail */}
            <div className="relative aspect-square bg-center bg-cover mb-2 overflow-hidden rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)] w-full" key={podcastEpisode.episode_id} style={{backgroundImage: `url("${selectPodcastImage(podcastEpisode)}")`}}>
                <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="material-symbols-outlined !text-7xl text-white group-active:scale-90" key={podcastEpisode.episode_id+"icon"}>play_circle</span>
                </div>

                {/* Progress bar */
                totalTime && watchedTime && (
                    <div className="absolute bottom-0 inset-x-0 bg-stone-900">
                        <div className="bg-[--accent-color] h-1.5" style={{width: (watchedTime/totalTime)*100+"%"}}></div>
                    </div>
                )}
            </div>

            {/* Titles */}
            <div>
                <span className="block font-bold leading-[1.2] mb-2 text-sm text-[--fg-color] transition-colors group-hover:text-[--fg-color-hover]">{podcastEpisode.name}</span>
                <span className="block leading-[1.2] text-xs text-[--fg-color]">{podcast.name}</span>
            </div>
        </div>
    )
}
