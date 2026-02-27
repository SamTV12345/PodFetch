import {FC, useEffect} from 'react'
import useOnMount from '../hooks/useOnMount'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { client } from '../utils/http'
import {getAudioPlayer} from "../utils/audioPlayer";

type HiddenAudioPlayerProps = {
    setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({ setAudioAmplifier }) => {
    const podcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)
    const setMetadata = useAudioPlayer(state => state.setMetadata)
    const setCurrentTimeUpdate = useAudioPlayer(state => state.setCurrentTimeUpdate)

    const setPlaying = useAudioPlayer(state => state.setPlaying)


    useEffect(() => {
        const audio = getAudioPlayer()
        if (!audio) return

        const onPlaying = () => {
            setPlaying(true)
        }
        const onPause = () => {
            setPlaying(false)
        }

        audio.addEventListener('playing', onPlaying)
        audio.addEventListener('pause', onPause)

        return () => {
            audio.removeEventListener('playing', onPlaying)
            audio.removeEventListener('pause', onPause)
        }
    }, [setPlaying])


    useOnMount(() => {
        if (/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)) {
            return
        }

        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        setAudioAmplifier(new AudioAmplifier(audioPlayer))
    })


    useEffect(() => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        const fallbackDuration = Number(podcastEpisode?.podcastEpisode.total_time || 0)
        const normalizeDuration = (duration: number) => {
            if (Number.isFinite(duration) && duration > 0) {
                return duration
            }
            return fallbackDuration > 0 ? fallbackDuration : 0
        }

        const updateMetadata = (el: HTMLMediaElement) => {
            setMetadata({
                currentTime: el.currentTime,
                duration: normalizeDuration(el.duration),
                percentage: 0
            })
        }

        const onTimeUpdate = (e: Event) => {
            const el = e.currentTarget as HTMLMediaElement
            setCurrentTimeUpdate(el.currentTime)
        }
        const onLoadedMetadata = (e: Event) => {
            const el = e.currentTarget as HTMLMediaElement
            updateMetadata(el)

            if (isNaN(el.duration)) {
                const currentEpisodeId = podcastEpisode?.podcastEpisode.episode_id
                if (!currentEpisodeId) {
                    return
                }
                client.GET('/api/v1/podcasts/episode/{id}', {
                    params: { path: { id: currentEpisodeId } }
                }).then((response) => {
                    setMetadata({
                        currentTime: el.currentTime,
                        duration: response.data?.total || normalizeDuration(el.duration),
                        percentage: 0
                    })
                })
            }
        }
        const onDurationChange = (e: Event) => {
            const el = e.currentTarget as HTMLMediaElement
            updateMetadata(el)
        }
        const onCanPlay = (e: Event) => {
            const el = e.currentTarget as HTMLMediaElement
            updateMetadata(el)
        }


        audioPlayer.addEventListener('timeupdate', onTimeUpdate)
        audioPlayer.addEventListener('loadedmetadata', onLoadedMetadata)
        audioPlayer.addEventListener('durationchange', onDurationChange)
        audioPlayer.addEventListener('canplay', onCanPlay)

        return () => {
            audioPlayer.removeEventListener('timeupdate', onTimeUpdate)
            audioPlayer.removeEventListener('loadedmetadata', onLoadedMetadata)
            audioPlayer.removeEventListener('durationchange', onDurationChange)
            audioPlayer.removeEventListener('canplay', onCanPlay)
        }
    }, [podcastEpisode?.podcastEpisode.episode_id, podcastEpisode?.podcastEpisode.total_time, setCurrentTimeUpdate, setMetadata]);

    return (
        <div></div>
    )
}
