import {FC, useEffect} from 'react'
import useOnMount from '../hooks/useOnMount'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { client } from '../utils/http'
import {getAudioPlayer} from "../utils/audioPlayer";
import useCommon from "../store/CommonSlice";
import {SKIPPED_TIME} from "../utils/Utilities";
import {logCurrentPlaybackTime} from "../utils/navigationUtils";

type HiddenAudioPlayerProps = {
    setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({ setAudioAmplifier }) => {
    const podcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)
    const currentPodcast = useAudioPlayer(state => state.currentPodcast)
    const currentPodcastEpisodeIndex = useAudioPlayer(state => state.currentPodcastEpisodeIndex)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setMetadata = useAudioPlayer(state => state.setMetadata)
    const setCurrentTimeUpdate = useAudioPlayer(state => state.setCurrentTimeUpdate)
    const selectedEpisodes = useCommon(state => state.selectedEpisodes)
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

    useEffect(() => {
        if (!('mediaSession' in navigator)) {
            return
        }

        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        const persistPlaybackTime = () => {
            if (!podcastEpisode) {
                return
            }
            logCurrentPlaybackTime(podcastEpisode.podcastEpisode.episode_id, audioPlayer.currentTime)
        }

        const switchToEpisodeAt = async (targetIndex: number) => {
            const targetEpisode = selectedEpisodes[targetIndex]
            if (!targetEpisode) {
                return
            }
            setCurrentPodcastEpisode(targetIndex)
            audioPlayer.pause()
            audioPlayer.src = targetEpisode.podcastEpisode.local_url
            audioPlayer.currentTime = targetEpisode.podcastHistoryItem?.position ?? 0
            audioPlayer.load()
            await audioPlayer.play()
        }

        const togglePlayPause = async () => {
            if (audioPlayer.paused) {
                await audioPlayer.play()
                return
            }
            persistPlaybackTime()
            audioPlayer.pause()
        }

        navigator.mediaSession.metadata = new MediaMetadata({
            title: podcastEpisode?.podcastEpisode.name ?? 'Podfetch',
            artist: currentPodcast?.name ?? 'Podcast',
            album: 'Podfetch',
            artwork: [
                {
                    src: podcastEpisode?.podcastEpisode.local_image_url || '/ui/default.jpg',
                    sizes: '512x512',
                    type: 'image/jpeg'
                }
            ]
        })

        navigator.mediaSession.playbackState = audioPlayer.paused ? 'paused' : 'playing'
        navigator.mediaSession.setActionHandler('play', () => {
            audioPlayer.play()
        })
        navigator.mediaSession.setActionHandler('pause', () => {
            persistPlaybackTime()
            audioPlayer.pause()
        })
        navigator.mediaSession.setActionHandler('seekbackward', (details) => {
            const seekOffset = details.seekOffset ?? SKIPPED_TIME
            audioPlayer.currentTime = Math.max(0, audioPlayer.currentTime - seekOffset)
        })
        navigator.mediaSession.setActionHandler('seekforward', (details) => {
            const seekOffset = details.seekOffset ?? SKIPPED_TIME
            const duration = Number.isFinite(audioPlayer.duration) ? audioPlayer.duration : Infinity
            audioPlayer.currentTime = Math.min(duration, audioPlayer.currentTime + seekOffset)
        })
        navigator.mediaSession.setActionHandler('seekto', (details) => {
            if (details.seekTime == null || !Number.isFinite(details.seekTime)) {
                return
            }
            audioPlayer.currentTime = details.seekTime
        })
        navigator.mediaSession.setActionHandler('previoustrack', () => {
            if (currentPodcastEpisodeIndex == null || currentPodcastEpisodeIndex <= 0) {
                return
            }
            void switchToEpisodeAt(currentPodcastEpisodeIndex - 1)
        })
        navigator.mediaSession.setActionHandler('nexttrack', () => {
            if (currentPodcastEpisodeIndex == null) {
                return
            }
            if (currentPodcastEpisodeIndex >= selectedEpisodes.length - 1) {
                return
            }
            void switchToEpisodeAt(currentPodcastEpisodeIndex + 1)
        })
        navigator.mediaSession.setActionHandler('stop', () => {
            void togglePlayPause()
        })

        return () => {
            navigator.mediaSession.setActionHandler('play', null)
            navigator.mediaSession.setActionHandler('pause', null)
            navigator.mediaSession.setActionHandler('seekbackward', null)
            navigator.mediaSession.setActionHandler('seekforward', null)
            navigator.mediaSession.setActionHandler('seekto', null)
            navigator.mediaSession.setActionHandler('previoustrack', null)
            navigator.mediaSession.setActionHandler('nexttrack', null)
            navigator.mediaSession.setActionHandler('stop', null)
        }
    }, [
        currentPodcastEpisodeIndex,
        currentPodcast?.name,
        podcastEpisode,
        selectedEpisodes,
        setCurrentPodcastEpisode
    ])


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
