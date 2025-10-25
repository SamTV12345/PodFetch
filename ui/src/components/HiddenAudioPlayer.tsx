import { FC, RefObject, useEffect } from 'react'
import useOnMount from '../hooks/useOnMount'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PodcastWatchedModel } from '../models/PodcastWatchedModel'
import { client } from '../utils/http'
import useCommon from "../store/CommonSlice";

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement|null>,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({ refItem, setAudioAmplifier }) => {
    const podcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)
    const setMetadata = useAudioPlayer(state => state.setMetadata)
    const setCurrentTimeUpdate = useAudioPlayer(state => state.setCurrentTimeUpdate)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const selectedEpisodes = useCommon(state => state.selectedEpisodes)

    useEffect(() => {
        if (podcastEpisode && refItem && refItem.current) {
            refItem.current.load()

            if (podcastEpisode.podcastHistoryItem!.position === undefined) {
                // fetch time from server

                client.GET("/api/v1/podcasts/episode/{id}", {
                    params: {
                        path:{
                            id: podcastEpisode.podcastEpisode.episode_id
                        }
                    }
                }).then((response) => {
                    const newSelectedEpisodes = selectedEpisodes.map((e)=>{
                        if (e.podcastEpisode.id === podcastEpisode.podcastEpisode.id) {
                            e.podcastHistoryItem = {
                                ...response.data!
                            }
                        }
                        return e
                    })
                    setSelectedEpisodes(newSelectedEpisodes)
                    refItem.current!.currentTime = podcastEpisode.podcastHistoryItem?.position!
                })

            } else {
                refItem.current!.currentTime = podcastEpisode.podcastHistoryItem?.position!
            }

            refItem.current.play()
        }
    },[podcastEpisode])

    useOnMount(() => {
        if (/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)) {
            return
        }

        setAudioAmplifier(new AudioAmplifier(refItem.current!))
    })

    return (
        <audio
            ref={refItem}
            crossOrigin="anonymous"
            src={podcastEpisode?.podcastEpisode.local_url}
            id={'hiddenaudio'}
            onTimeUpdate={(e) => {
               setCurrentTimeUpdate(e.currentTarget.currentTime)
            }}
            onLoadedMetadata={(e) => {
                setMetadata({
                    currentTime: e.currentTarget.currentTime,
                    duration: e.currentTarget.duration,
                    percentage: 0
                })

                if (isNaN(e.currentTarget.duration)) {
                    // Need alternative method of getting duration
                    // Firefox doesn't load the entire file before playing
                    // causing a changing duration, but the onLoadedMetadata event
                    // is only called once rendering the progressbar useless
                    client.GET("/api/v1/podcasts/episode/{id}", {
                        params: {
                            path: {
                                id: podcastEpisode!.podcastEpisode.episode_id
                            }
                        }
                    }).then((response) => {
                        setMetadata({
                            currentTime: e.currentTarget.currentTime,
                            duration: response.data!.total!,
                            percentage: 0
                        })
                    })

                }
            }}
        />
    )
}
