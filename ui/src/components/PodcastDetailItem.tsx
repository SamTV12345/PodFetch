import {FC, Fragment, useMemo} from 'react'
import { useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Waypoint } from 'react-waypoint'
import { useSnackbar } from 'notistack'
import {formatTime, prependAPIKeyOnAuthEnabled, removeHTML} from '../utils/Utilities'
import 'material-symbols/outlined.css'
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";
import useCommon from "../store/CommonSlice";
import {Episode} from "../models/Episode";
import {handlePlayofEpisode} from "../utils/PlayHandler";
import {logCurrentPlaybackTime} from "../utils/navigationUtils";
import {components} from "../../schema";
import {client} from "../utils/http";

type PodcastDetailItemProps = {
    episode: components["schemas"]["PodcastEpisodeWithHistory"],
    index: number,
    episodesLength: number,
    onlyUnplayed: boolean
}

export const PodcastDetailItem: FC<PodcastDetailItemProps> = ({ episode, index,episodesLength, onlyUnplayed}) => {
    const params = useParams()
    const { enqueueSnackbar } = useSnackbar()
    const { t } =  useTranslation()
    const selectedEpisodes = useCommon(state => state.selectedEpisodes)
    const percentagePlayed = useMemo(()=>{
        if(!episode.podcastHistoryItem){
            return -1
        }
        return Math.round(episode.podcastHistoryItem.position!*100/episode.podcastEpisode.total_time)
    }, [episode.podcastHistoryItem?.position])
    const addPodcastEpisodes = useCommon(state => state.addPodcastEpisodes)
    const setEpisodeDownloaded = useCommon(state => state.setEpisodeDownloaded)
    const setInfoModalPodcast = useCommon(state => state.setInfoModalPodcast)
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)

    const playedTime = useMemo(()=>{
        if(percentagePlayed === -1){
            return t('not-yet-played')
        }
        return t('podcast-episode-played',{
            percentage: percentagePlayed+"%"
        })
    },[percentagePlayed])

    return (
        <Fragment key={'episode_' + episode.podcastEpisode.id}>
            <div id={'episode_' + episode.podcastEpisode.id} className="
                grid
                grid-cols-[1fr_auto] grid-rows-[auto_auto_auto]
                xs:grid-cols-[auto_1fr_auto]
                gap-x-4 gap-y-0 xs:gap-y-2
                items-center group cursor-pointer mb-12
            ">
                {/* Thumbnail */}
                <img src={episode.podcastEpisode.local_image_url} alt={episode.podcastEpisode.name} className="
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
                    <span className="text-sm text-(--fg-secondary-color)">{formatTime(episode.podcastEpisode.date_of_recording)}</span>
                    <span className="text-sm text-(--fg-secondary-color)">{playedTime}</span>

                    <span className="flex gap-5">
                    <span title={t('download-to-server') as string} className={`material-symbols-outlined text-(--fg-icon-color)
                     ${episode.podcastEpisode.status ? 'cursor-auto filled' : 'cursor-pointer hover:text-(--fg-icon-color-hover)'}`} onClick={(e)=>{
                        // Prevent icon click from triggering info modal
                        e.stopPropagation()

                        // Prevent another download if already downloaded
                        if (episode.podcastEpisode.status) {
                            return
                        }

                        client.PUT("/api/v1/podcasts/{id}/episodes/download", {
                            params: {
                                path: {
                                    id: episode.podcastEpisode.episode_id
                                }
                            }
                        }).then(()=>{
                            enqueueSnackbar(t('episode-downloaded-to-server'), {variant: "success"})
                            setEpisodeDownloaded(episode.podcastEpisode.episode_id)
                        })
                    }}>cloud_download</span>
                        {/* Check icon */}
                        <span className="material-symbols-outlined text-(--fg-icon-color) active:scale-95" onClick={(e)=>{
                            // Prevent icon click from triggering info modal
                            e.stopPropagation()
                            logCurrentPlaybackTime(episode.podcastEpisode.episode_id, episode.podcastHistoryItem?.total || 0)
                            const mappedEpisodes = selectedEpisodes.map(s=>{
                                if (s.podcastEpisode.episode_id === episode.podcastEpisode.episode_id){
                                    if (s.podcastHistoryItem) {
                                        s.podcastHistoryItem.position = episode.podcastEpisode.total_time
                                    } else {
                                        s.podcastHistoryItem = {
                                            action: "new",
                                            device: "",
                                            episode: "",
                                            guid: "",
                                            podcast: "",
                                            started: 0,
                                            timestamp: "",
                                            total: episode.podcastEpisode.total_time,
                                            position: episode.podcastEpisode.total_time
                                        }
                                    }

                                }
                                return s
                            })
                            setSelectedEpisodes(mappedEpisodes)
                        }}>check</span>
                        <span className={"material-symbols-outlined text-(--fg-color) " + (episode.podcastEpisode.favored && 'filled')}
                              onClick={() => {
                                  client.PUT(  "/api/v1/podcasts/{id}/episodes/favor", {
                                      params: {
                                          path: {
                                                id: episode.podcastEpisode.id
                                          }
                                      },
                                      body: {
                                          favored: !episode.podcastEpisode.favored
                                      }
                                  })
                                      .then(() => {
                                            const mappedEpisodes = selectedEpisodes.map(s => {
                                                if (s.podcastEpisode.episode_id === episode.podcastEpisode.episode_id) {
                                                    s.podcastEpisode.favored = !episode.podcastEpisode.favored
                                                }
                                                return s
                                            })
                                            setSelectedEpisodes(mappedEpisodes)
                                        })
                              }}
                        >favorite</span>
                    </span>
                </div>

                {/* Title */}
                <span className="
                    col-start-1 col-end-2 row-start-2 row-end-3
                    xs:col-start-2 xs:col-end-3
                    font-bold leading-tight  text-(--fg-color) transition-color group-hover:text-(--fg-color-hover)
                "  onClick={() => {
                    setInfoModalPodcast(episode.podcastEpisode)
                    setInfoModalPodcastOpen(true)
                }}>{episode.podcastEpisode.name}</span>

                {/* Description */}
                <div className="
                    line-clamp-3
                    col-start-1 col-end-3 row-start-3 row-end-4
                    xs:col-start-2 xs:col-end-3
                    leading-[1.75] text-sm text-(--fg-color) transition-color group-hover:text-(--fg-color-hover)
                "  onClick={() => {
                    setInfoModalPodcast(episode.podcastEpisode)
                    setInfoModalPodcastOpen(true)
                }} dangerouslySetInnerHTML={removeHTML(episode.podcastEpisode.description)}></div>

                {/* Play button */}
                <span className={`${percentagePlayed >=95  && episode.podcastEpisode.total_time > 0 && 'text-gray-500'}
                    col-start-2 col-end-3 row-start-2 row-end-3
                    xs:col-start-3 xs:col-end-4 xs:row-start-1 xs:row-end-4
                    self-center material-symbols-outlined cursor-pointer !text-5xl text-(--fg-color) hover:text-(--fg-color-hover) active:scale-90
                `} key={episode.podcastEpisode.episode_id + 'icon'} onClick={(e) => {
                    // Prevent icon click from triggering info modal
                    e.stopPropagation()
                    client.GET("/api/v1/podcasts/episode/{id}", {
                        params: {
                            path: {
                                id: episode.podcastEpisode.episode_id
                            }
                        }
                    }).then((resp)=>{
                        handlePlayofEpisode(episode.podcastEpisode, resp.data!,)
                    }).catch(e=>{
                        handlePlayofEpisode(episode.podcastEpisode, undefined)
                    })
                }}>play_circle</span>
            </div>

            {/* Infinite scroll */
            index === (episodesLength - 5) &&
                <Waypoint key={index + 'waypoint'} onEnter={() => {
                    client.GET("/api/v1/podcasts/{id}/episodes", {
                        params: {
                            path: {
                                id: params.id!,
                            },
                            query: {
                                last_podcast_episode: selectedEpisodes[selectedEpisodes.length - 1]!.podcastEpisode.date_of_recording,
                                only_unlistened: onlyUnplayed
                            }
                        }
                    })
                        .then((response) => {
                            addPodcastEpisodes(response.data!)
                        })
                }} />
            }
        </Fragment>
    )
}
