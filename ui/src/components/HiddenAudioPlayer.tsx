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

    const setPlaying = useAudioPlayer(state => state.setPlaying)


    useEffect(() => {
        const audio = refItem.current
        if (!audio) return

        const onPlaying = () => {
            setPlaying(true)
        }
        const onPause = () => {
            setPlaying(false)
        }

        audio.addEventListener('play', onPlaying)
        audio.addEventListener('pause', onPause)

        return () => {
            audio.removeEventListener('playing', onPlaying)
            audio.removeEventListener('pause', onPause)
        }
    }, [refItem, setPlaying])


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
