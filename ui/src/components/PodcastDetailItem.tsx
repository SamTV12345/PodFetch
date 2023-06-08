import {FC} from "react"
import {Waypoint} from "react-waypoint"
import {useParams} from "react-router-dom"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {apiURL, formatTime, prepareOnlinePodcastEpisode, preparePodcastEpisode, removeHTML} from "../utils/Utilities"
import {store} from "../store/store"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {addPodcastEpisodes, PodcastEpisode, setInfoModalDownloaded, setInfoModalPodcast, setInfoModalPodcastOpen} from "../store/CommonSlice"
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice"
import {PodcastWatchedModel} from "../models/PodcastWatchedModel"
import "material-symbols/outlined.css"

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
    const {t} =  useTranslation()

    if (currentPodcast === undefined) {
        return <div>"Nicht gefunden"</div>
    }

    return <>
        <div key={episode.episode_id} id={"episode_" + episode.id} className="grid grid-cols-[auto_1fr_auto] gap-x-4 group cursor-pointer mb-12" onClick={() => {
            dispatch(setInfoModalPodcast(episode))
            dispatch(setInfoModalPodcastOpen(true))
        }}>
            <img src={currentPodcast.image_url} alt={currentPodcast.name} className="row-span-2 self-center rounded-lg w-32 transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)]"/>

            <div className="self-center grid grid-cols-[7rem_1fr] grid-rows-[auto_auto_auto] gap-x-4 gap-y-2">
                <span className="text-sm text-stone-500">{formatTime(episode.date_of_recording)}</span>

                <span title={t('download-to-server') as string} className={`material-symbols-outlined text-stone-800 ${episode.status === 'D' ? 'cursor-auto filled' : 'cursor-pointer hover:text-stone-600'}`} onClick={(e)=>{
                    // Prevent icon click from triggering info modal
                    e.stopPropagation()

                    axios.put(apiURL + "/podcast/" + episode.episode_id + "/episodes/download")
                        .then(()=>{
                            dispatch(setInfoModalDownloaded(episode.episode_id))
                        })
                }}>cloud_download</span>


                <span className="col-span-2 font-bold leading-tight text-stone-900 transition-color group-hover:text-stone-600">{episode.name}</span>

                <div className="col-span-2 line-clamp-3 leading-[1.75] text-sm text-stone-900 transition-color group-hover:text-stone-600" dangerouslySetInnerHTML={removeHTML(episode.description)}></div>
            </div>

            <span className="self-center material-symbols-outlined cursor-pointer !text-5xl text-stone-900 hover:text-stone-600 active:scale-90" key={episode.episode_id + "icon"} onClick={(e) => {
                // Prevent icon click from triggering info modal
                e.stopPropagation()

                axios.get(apiURL + "/podcast/episode/" + episode.episode_id)
                    .then((response: AxiosResponse<PodcastWatchedModel>) => {
                        episode.status === 'D' ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(episode, response.data))) : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode, response.data)))
                        dispatch(setCurrentPodcast(currentPodcast))
                        dispatch(setPlaying(true))
                    })
            }}>play_circle</span>
        </div>

        {/* Infinite scroll */
            index===episodesLength-5&&<Waypoint key={index+"waypoint"} onEnter={()=>{
                axios.get(apiURL+"/podcast/"+params.id+"/episodes?last_podcast_episode="+episode.date_of_recording)
                    .then((response)=>{
                        dispatch(addPodcastEpisodes(response.data))
                    })
            }}/>
        }
    </>
}
