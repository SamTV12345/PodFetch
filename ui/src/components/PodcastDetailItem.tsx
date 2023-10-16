import {FC, useMemo} from 'react'
import { useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Waypoint } from 'react-waypoint'
import axios, { AxiosResponse } from 'axios'
import { useSnackbar } from 'notistack'
import { store } from '../store/store'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import {
    addPodcastEpisodes,
    setEpisodeDownloaded,
    setInfoModalPodcast,
    setInfoModalPodcastOpen,
    setPodcastAlreadyPlayed, setPodcastEpisodeAlreadyPlayed
} from '../store/CommonSlice'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { apiURL, formatTime, prepareOnlinePodcastEpisode, preparePodcastEpisode, removeHTML } from '../utils/Utilities'
import { PodcastWatchedModel } from '../models/PodcastWatchedModel'
import 'material-symbols/outlined.css'
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";

type PodcastDetailItemProps = {
    episode: EpisodesWithOptionalTimeline,
    index: number,
    episodesLength: number
}

export const PodcastDetailItem: FC<PodcastDetailItemProps> = ({ episode, index,episodesLength }) => {
    const dispatch = useAppDispatch()
    const currentPodcast = useAudioPlayer(state => state.currentPodcast)
    const params = useParams()
    const { enqueueSnackbar } = useSnackbar()
    const { t } =  useTranslation()
    const selectedEpisodes = useAppSelector(state => state.common.selectedEpisodes)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const setPlaying = useAudioPlayer(state => state.setPlaying)
    const percentagePlayed = useMemo(()=>{
        if(!episode.podcastHistoryItem){
            return -1
        }
        return Math.round(episode.podcastHistoryItem.watchedTime*100/episode.podcastEpisode.total_time)
    }, [episode.podcastHistoryItem?.watchedTime])

    const playedTime = useMemo(()=>{
        if(percentagePlayed === -1){
            return t('not-yet-played')
        }
        return t('podcast-episode-played',{
            percentage: percentagePlayed+"%"
        })
    },[percentagePlayed])

    return (
        <>
            <div id={'episode_' + episode.podcastEpisode.id} className="
                grid
                grid-cols-[1fr_auto] grid-rows-[auto_auto_auto]
                xs:grid-cols-[auto_1fr_auto]
                gap-x-4 gap-y-0 xs:gap-y-2
                items-center group cursor-pointer mb-12
            " onClick={() => {
                dispatch(setInfoModalPodcast(episode.podcastEpisode))
                dispatch(setInfoModalPodcastOpen(true))
            }}>
                {/* Thumbnail */}
                <img src={episode.podcastEpisode.image_url} alt={episode.podcastEpisode.name} className="
                    hidden xs:block
                    col-start-1 col-end-2 row-start-1 row-end-4
                    self-center rounded-lg w-32 transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)]
                "/>

                {/* Date and download icon */}
                <div className="
                    col-start-1 col-end-3 row-start-1-row-end-2
                    xs:col-start-2 xs:col-end-3
                    self-center
                    grid grid-cols-[7rem_1fr_3fr] gap-x-4 items-center
                ">
                    <span className="text-sm text-[--fg-secondary-color]">{formatTime(episode.podcastEpisode.date_of_recording)}</span>
                    <span className="text-sm text-[--fg-secondary-color]">{playedTime}</span>

                    <span title={t('download-to-server') as string} className={`material-symbols-outlined text-[--fg-icon-color]
                     ${episode.podcastEpisode.status === 'D' ? 'cursor-auto filled' : 'cursor-pointer hover:text-[--fg-icon-color-hover]'}`} onClick={(e)=>{
                        // Prevent icon click from triggering info modal
                        e.stopPropagation()

                        // Prevent another download if already downloaded
                        if (episode.podcastEpisode.status === 'D') {
                            return
                        }

                        axios.put(apiURL + "/podcast/" + episode.podcastEpisode.episode_id + "/episodes/download")
                            .then(()=>{
                                enqueueSnackbar(t('episode-downloaded-to-server'), {variant: "success"})
                                dispatch(setEpisodeDownloaded(episode.podcastEpisode.episode_id))
                            })
                    }}>cloud_download</span>
                </div>

                {/* Title */}
                <span className="
                    col-start-1 col-end-2 row-start-2 row-end-3
                    xs:col-start-2 xs:col-end-3
                    font-bold leading-tight  text-[--fg-color] transition-color group-hover:text-[--fg-color-hover]
                ">{episode.podcastEpisode.name}</span>

                {/* Description */}
                <div className="
                    line-clamp-3
                    col-start-1 col-end-3 row-start-3 row-end-4
                    xs:col-start-2 xs:col-end-3
                    leading-[1.75] text-sm text-[--fg-color] transition-color group-hover:text-[--fg-color-hover]
                " dangerouslySetInnerHTML={removeHTML(episode.podcastEpisode.description)}></div>

                {/* Play button */}
                <span className={`${percentagePlayed >=95  && episode.podcastEpisode.total_time > 0 && 'text-gray-500'}
                    col-start-2 col-end-3 row-start-2 row-end-3
                    xs:col-start-3 xs:col-end-4 xs:row-start-1 xs:row-end-4
                    self-center material-symbols-outlined cursor-pointer !text-5xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90
                `} key={episode.podcastEpisode.episode_id + 'icon'} onClick={(e) => {
                    // Prevent icon click from triggering info modal
                    e.stopPropagation()

                    axios.get(apiURL + '/podcast/episode/' + episode.podcastEpisode.episode_id)
                        .then((response: AxiosResponse<PodcastWatchedModel>) => {
                            const playedPercentage = response.data.watchedTime * 100 / episode.podcastEpisode.total_time
                            if(playedPercentage < 95 || episode.podcastEpisode.total_time === 0){
                                episode.podcastEpisode.status === 'D'
                                    ? setCurrentPodcastEpisode(preparePodcastEpisode(episode.podcastEpisode, response.data))
                                    : setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode.podcastEpisode, response.data))

                                currentPodcast && setCurrentPodcast(currentPodcast)
                                setPlaying(true)
                            }
                            else{
                                dispatch(setPodcastEpisodeAlreadyPlayed({
                                    podcastEpisode: episode,
                                    podcastWatchModel: response.data
                                }))
                                dispatch(setPodcastAlreadyPlayed(true))
                            }
                        })
                }}>play_circle</span>
            </div>

            {/* Infinite scroll */
            index === (episodesLength - 5) &&
                <Waypoint key={index + 'waypoint'} onEnter={() => {
                    axios.get(apiURL + '/podcast/' + params.id + '/episodes',{
                        params: {
                            last_podcast_episode: selectedEpisodes[selectedEpisodes.length - 1].podcastEpisode.date_of_recording
                        }
                    })
                        .then((response:AxiosResponse<EpisodesWithOptionalTimeline[]>) => {
                            dispatch(addPodcastEpisodes(response.data))
                        })
                }} />
            }
        </>
    )
}
