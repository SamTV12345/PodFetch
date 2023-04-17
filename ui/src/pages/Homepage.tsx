import {useEffect, useState} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL, prepareOnlinePodcastEpisode, preparePath, preparePodcastEpisode} from "../utils/Utilities";
import {PodcastWatchedEpisodeModel} from "../models/PodcastWatchedEpisodeModel";
import {PlayIcon} from "../components/PlayIcon";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {store} from "../store/store";
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {useAppDispatch} from "../store/hooks";
import {useTranslation} from "react-i18next";

export const Homepage = () => {
    const [podcastWatched, setPodcastWatched] = useState<PodcastWatchedEpisodeModel[]>([])
    const dispatch = useAppDispatch()
    const {t} = useTranslation()



    useEffect(()=>{
            axios.get(apiURL+"/podcast/episode/lastwatched")
                .then((v:AxiosResponse<PodcastWatchedEpisodeModel[]>)=>{
                    setPodcastWatched(v.data)
                })

    }, [])

    const selectPodcastImage = (podcast: PodcastWatchedEpisodeModel) => {
        if(podcast.podcastEpisode.local_image_url.length>1){
            return preparePath(podcast.podcastEpisode.local_image_url)
        }
        else{
            return podcast.podcastEpisode.image_url
        }
    }

    return <div className="p-3">
        <h1 className="font-bold text-2xl">{t('last-listened')}</h1>
        <div className="grid grid-cols-2 md:grid-cols-5 xs:grid-cols-1 gap-4">
        {
            podcastWatched.map((v)=>{
                return <div key={v.episodeId}
                    className="max-w-sm rounded-lg shadow bg-gray-800 border-gray-700">
                    <div className="relative" key={v.episodeId}>
                        <img src={selectPodcastImage(v)} alt="" className=""/>
                        <div className="absolute left-0 top-0 w-full h-full hover:bg-gray-500 opacity-80 z-10 grid place-items-center play-button-background">
                            <PlayIcon key={v.podcastEpisode.episode_id+"icon"} podcast={v.podcastEpisode} className="w-20 h-20 opacity-0" onClick={()=>{
                                axios.get(apiURL+"/podcast/episode/"+v.podcastEpisode.episode_id)
                                    .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                                        if (v.podcastEpisode.local_image_url.trim().length>1){
                                            store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(v.podcastEpisode, response.data)))
                                        }
                                        else{
                                            store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(v.podcastEpisode, response.data)))
                                        }


                                        dispatch(setCurrentPodcast(v.podcast))
                                        dispatch(setPlaying(true))
                                    })
                            }}/>
                        </div>
                    </div>
                    <div className="relative border-box w-11/12">
                        <div className="bg-blue-900 h-2" style={{width: (v.watchedTime/v.totalTime)*100+"%"}}></div>
                    </div>
                    <div className="p-5">
                            <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white break-words">{v.name}</h5>
                    </div>
                </div>
            })
        }
        </div>
    </div>
}
