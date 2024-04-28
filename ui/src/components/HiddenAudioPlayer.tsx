import { FC, RefObject, useEffect } from 'react'
import axios, { AxiosResponse } from 'axios'
import useOnMount from '../hooks/useOnMount'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PodcastWatchedModel } from '../models/PodcastWatchedModel'

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({ refItem, setAudioAmplifier }) => {
    const podcastEpisode = useAudioPlayer(state => state.currentPodcastEpisode)
    const setMetadata = useAudioPlayer(state => state.setMetadata)
    const setCurrentTimeUpdate = useAudioPlayer(state => state.setCurrentTimeUpdate)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)

    useEffect(() => {
        if (podcastEpisode && refItem && refItem.current) {
            refItem.current.load()

            if (podcastEpisode.time === undefined) {
                // fetch time from server
                axios.get(  '/podcast/episode/' + podcastEpisode.episode_id)
                    .then((response: AxiosResponse<PodcastWatchedModel>) => {
                        setCurrentPodcastEpisode({
                            ...podcastEpisode,
                            time: response.data.position
                        })
                        refItem.current!.currentTime = podcastEpisode.time!
                    })
            } else {
                refItem.current!.currentTime = podcastEpisode.time
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
            src={podcastEpisode?.local_url}
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
                     axios.get('/podcast/episode/' + podcastEpisode!.episode_id)
                       .then((response: AxiosResponse<PodcastWatchedModel>) => {
                           setMetadata({
                               currentTime: e.currentTarget.currentTime,
                               duration: response.data.total,
                               percentage: 0
                           })
                       })
                }
            }}
        />
    )
}
