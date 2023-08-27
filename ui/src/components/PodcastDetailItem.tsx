import { FC } from 'react'
import { useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Waypoint } from 'react-waypoint'
import axios, { AxiosResponse } from 'axios'
import { useSnackbar } from 'notistack'
import { store } from '../store/store'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { addPodcastEpisodes, PodcastEpisode, setEpisodeDownloaded, setInfoModalPodcast, setInfoModalPodcastOpen } from '../store/CommonSlice'
import { setCurrentPodcast, setCurrentPodcastEpisode, setPlaying } from '../store/AudioPlayerSlice'
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
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)
    const params = useParams()
    const { enqueueSnackbar } = useSnackbar()
    const { t } =  useTranslation()
    const selectedEpisodes = useAppSelector(state => state.common.selectedEpisodes)
    return (
        <>
            <div key={episode.podcastEpisode.episode_id} id={'episode_' + episode.podcastEpisode.id} className="
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
                    grid grid-cols-[7rem_1fr] gap-x-4 items-center
                ">
                    <span className="text-sm text-[--fg-secondary-color]">{formatTime(episode.podcastEpisode.date_of_recording)}</span>

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
                    font-bold leading-tight text-[--fg-color] transition-color group-hover:text-[--fg-color-hover]
                ">{episode.podcastEpisode.name}</span>

                {/* Description */}
                <div className="
                    line-clamp-3
                    col-start-1 col-end-3 row-start-3 row-end-4
                    xs:col-start-2 xs:col-end-3
                    leading-[1.75] text-sm text-[--fg-color] transition-color group-hover:text-[--fg-color-hover]
                " dangerouslySetInnerHTML={removeHTML(episode.podcastEpisode.description)}></div>

                {/* Play button */}
                <span className="
                    col-start-2 col-end-3 row-start-2 row-end-3
                    xs:col-start-3 xs:col-end-4 xs:row-start-1 xs:row-end-4
                    self-center material-symbols-outlined cursor-pointer !text-5xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90
                " key={episode.podcastEpisode.episode_id + 'icon'} onClick={(e) => {
                    // Prevent icon click from triggering info modal
                    e.stopPropagation()

                    axios.get(apiURL + '/podcast/episode/' + episode.podcastEpisode.episode_id)
                        .then((response: AxiosResponse<PodcastWatchedModel>) => {
                            episode.podcastEpisode.status === 'D'
                            ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(episode.podcastEpisode, response.data)))
                            : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode.podcastEpisode, response.data)))

                            currentPodcast&& dispatch(setCurrentPodcast(currentPodcast))
                            dispatch(setPlaying(true))
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
