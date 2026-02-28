import {FC, useEffect, useMemo} from 'react'
import {
    SKIPPED_TIME
} from '../utils/Utilities'
import useAudioPlayer, {type AudioPlayerPlay} from '../store/AudioPlayerSlice'
import 'material-symbols/outlined.css'
import { useKeyDown } from '../hooks/useKeyDown'
import useCommon from "../store/CommonSlice";
import {getAudioPlayer, startAudioPlayer} from "../utils/audioPlayer";
import {cn} from "../lib/utils";
import {usePlaybackLogger} from "../hooks/usePlaybackLogger";

type PlayerTimeControlsProps = {
    currentPodcastEpisode?: AudioPlayerPlay
}


const SPEED_STEPS = [0.5, 1,1.1,1.25, 1.5, 2, 2.5, 3]


export const PlayerTimeControls: FC<PlayerTimeControlsProps> = ({ currentPodcastEpisode }) => {
    const logCurrentPlaybackTime = usePlaybackLogger()
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const episodes = useCommon(state => state.selectedEpisodes)
    const isPlaying  = useAudioPlayer(state => state.isPlaying)
    const speed = useAudioPlayer(state => state.playBackRate)
    const time  = useAudioPlayer(state => state.metadata?.currentTime)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setPlaybackRate = useAudioPlayer(state => state.setPlayBackRate)

    const [hasPrevious, index, hasNext] = useMemo(()=>{
        if (!currentPodcastEpisode) {
            return [false, -1, false]
        }
        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode?.podcastEpisode.id)

        return [index > 0, index, index < episodes.length -1]
    }, [episodes, currentPodcastEpisode])


    const skipToPreviousEpisode = () => {
        if (currentPodcastEpisode === undefined) return


        if (index === -1) return

        if (index === 0) return

        switchToEpisodes(index - 1)
    }

    useEffect(() => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }
        audioPlayer.onended = () => {
            if (currentPodcastEpisode === undefined) return
            logCurrentPlaybackTime(currentPodcastEpisode!.podcastEpisode.episode_id, currentPodcastEpisode!.podcastEpisode.total_time)
        }
    }, [currentPodcastEpisode]);


    const skipToNextEpisode = () => {
        if (currentPodcastEpisode === undefined) return

        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode.podcastEpisode.id)

        if (index === -1) return

        if (index === episodes.length + 1) return

        switchToEpisodes(index + 1)
    }

    const switchToEpisodes = async (index: number) => {
        const nextEpisode = episodes[index]
        if (!nextEpisode) return
        setCurrentPodcastEpisode(index)
        await startAudioPlayer(nextEpisode.podcastEpisode.local_url, nextEpisode.podcastHistoryItem?.position ?? 0)
    }

    const handleButton = () => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        if (audioPlayer.paused) {
            void audioPlayer.play().catch(() => {})
        } else {
            if (time && currentPodcastEpisode) {
                logCurrentPlaybackTime(currentPodcastEpisode.podcastEpisode.episode_id, time)
                const mappedEpisodes = episodes.map(e=>{
                    if(e.podcastEpisode.episode_id === currentPodcastEpisode.podcastEpisode.episode_id){
                        return {
                            ...e,
                           podcastHistoryItem:{
                                 ...e.podcastHistoryItem!,
                               position: time
                           }
                        }
                    }
                    return e
                })
                setSelectedEpisodes(mappedEpisodes)
            }

            audioPlayer.pause()
        }
    }

    const changeSpeed = () => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        const currentIndex = SPEED_STEPS.indexOf(speed)

        if (currentIndex === SPEED_STEPS.length - 1) {
            audioPlayer.playbackRate = SPEED_STEPS[0]!
            setPlaybackRate(SPEED_STEPS[0]!)
            return
        }

        audioPlayer.playbackRate = SPEED_STEPS[currentIndex + 1]!
        setPlaybackRate(SPEED_STEPS[currentIndex + 1]!)
    }

    const seekForward = () => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        if (audioPlayer.currentTime + SKIPPED_TIME < audioPlayer.duration) {
            audioPlayer.currentTime += SKIPPED_TIME
        }
    }

    const seekBackward = () => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }

        if (audioPlayer.currentTime - SKIPPED_TIME > 0 ) {
            audioPlayer.currentTime -= SKIPPED_TIME
        } else {
            audioPlayer.currentTime = 0
        }
    }

    useKeyDown(seekBackward, ['j', 'ArrowLeft'], false)

    useKeyDown(seekForward, ['l', 'ArrowRight'], false)

    useKeyDown(handleButton, ['k', ' '], false)

    return (
        <div className="flex items-center justify-center gap-6">
            {/* Skip back */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl ui-text hover:ui-text-hover active:scale-90 " onClick={() => seekBackward()}>replay_30</span>

            {/* Previous */}
            <button disabled={!hasPrevious} className={cn("material-symbols-outlined filled text-3xl lg:text-4xl ui-text hover:ui-text-hover active:scale-90", hasPrevious ? '':'opacity-10')} onClick={() => skipToPreviousEpisode()}>skip_previous</button>

            {/* Play/pause */}
            <span className="flex items-center justify-center ui-bg-foreground hover:bg-(--fg-color-hover) cursor-pointer h-10 w-10 lg:h-12 lg:w-12 rounded-full active:scale-90" onClick={() => handleButton()}>
                {isPlaying?
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl ui-text-inverse">pause</span>:
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl ui-text-inverse">play_arrow</span>
                }
            </span>

            {/* Next */}
            <button disabled={!hasNext} className={cn("material-symbols-outlined filled text-3xl lg:text-4xl ui-text hover:ui-text-hover active:scale-90", hasNext ? '':'opacity-10')} onClick={() => skipToNextEpisode()}>skip_next</button>

            {/* Skip forward */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl ui-text hover:ui-text-hover active:scale-90" onClick={() => seekForward()}>forward_30</span>

            {/* Speed fixed width to prevent layout shift when value changes */}
            <span className="cursor-pointer text-sm ui-text hover:ui-text-hover w-8" onClick={() => changeSpeed()}>{speed}x</span>
        </div>
    )
}
