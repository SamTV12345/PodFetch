import { FC, RefObject, useEffect } from 'react'
import axios, { AxiosResponse } from 'axios'
import useOnMount from '../hooks/useOnMount'
import { store } from '../store/store'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { setCurrentPodcastEpisode, setCurrentTimeUpdate, setMetadata } from '../store/AudioPlayerSlice'
import { apiURL } from '../utils/Utilities'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PodcastWatchedModel } from '../models/PodcastWatchedModel'

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({ refItem, setAudioAmplifier }) => {
    const dispatch = useAppDispatch()
    const podcastEpisode = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)

    useEffect(() => {
        if (podcastEpisode && refItem && refItem.current) {
            refItem.current.load()

            if (podcastEpisode.time === undefined) {
                // fetch time from server
                axios.get(apiURL + '/podcast/episode/' + podcastEpisode.episode_id)
                    .then((response: AxiosResponse<PodcastWatchedModel>) => {
                        store.dispatch(setCurrentPodcastEpisode({
                            ...podcastEpisode,
                            time: response.data.watchedTime
                        }))
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
                dispatch(setCurrentTimeUpdate(e.currentTarget.currentTime))
            }}
            onLoadedMetadata={(e) => {
                dispatch(setMetadata({
                    currentTime: e.currentTarget.currentTime,
                    duration: e.currentTarget.duration,
                    percentage: 0
                }))
            }}
        />
    )
}
