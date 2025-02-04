import {FC, RefObject, useEffect} from 'react'
import {
    prepareOnlinePodcastEpisode,
    preparePodcastEpisode,
    SKIPPED_TIME
} from '../utils/Utilities'
import useAudioPlayer from '../store/AudioPlayerSlice'
import 'material-symbols/outlined.css'
import { useKeyDown } from '../hooks/useKeyDown'
import useCommon from "../store/CommonSlice";
import {logCurrentPlaybackTime} from "../utils/navigationUtils";
import {client} from "../utils/http";

type PlayerTimeControlsProps = {
    refItem: RefObject<HTMLAudioElement|null>
}


const SPEED_STEPS = [0.5, 1,1.1,1.25, 1.5, 2, 2.5, 3]


export const PlayerTimeControls: FC<PlayerTimeControlsProps> = ({ refItem }) => {
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const currentPodcastEpisode = useAudioPlayer(state => state.currentPodcastEpisode)
    const episodes = useCommon(state => state.selectedEpisodes)
    const isPlaying  = useAudioPlayer(state => state.isPlaying)
    const speed = useAudioPlayer(state => state.playBackRate)
    const time  = useAudioPlayer(state => state.metadata?.currentTime)
    const selectedEpisodes = useCommon(state => state.selectedEpisodes)
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setPlaying = useAudioPlayer(state => state.setPlaying)
    const setPlaybackRate = useAudioPlayer(state => state.setPlayBackRate)

    const skipToPreviousEpisode = () => {
        if (currentPodcastEpisode === undefined) return

        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode.podcastEpisode.id)

        if (index === -1) return

        if (index === 0) return

        switchToEpisodes(index - 1)
    }

    useEffect(() => {
        refItem.current!.onended = () => {
            logCurrentPlaybackTime(useAudioPlayer.getState().currentPodcastEpisode!.podcastEpisode.episode_id,
                useAudioPlayer.getState().currentPodcastEpisode!.podcastEpisode.total_time)
        }
    }, []);


    const skipToNextEpisode = () => {
        if (currentPodcastEpisode === undefined) return

        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode.podcastEpisode.id)

        if (index === -1) return

        if (index === episodes.length + 1) return

        switchToEpisodes(index + 1)
    }

    const switchToEpisodes = (index: number) => {
        if (refItem === undefined || refItem.current === undefined|| refItem.current === null) return

        const nextEpisode = episodes[index]!

        client.GET("/api/v1/podcasts/episode/{id}", {
            params: {
                path: {
                    id: nextEpisode.podcastEpisode.episode_id
                }
            }
        }) .then((response) => {
            setCurrentPodcastEpisode(nextEpisode)
            nextEpisode.podcastEpisode.status
                ? setCurrentPodcastEpisode(preparePodcastEpisode(nextEpisode.podcastEpisode, response.data!))
                : setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(nextEpisode.podcastEpisode, response.data!))
            refItem.current!.src = episodes[index]!.podcastEpisode.local_url
            refItem.current!.load()
            refItem.current?.play()
            setPlaying(true)
        })
    }

    const handleButton = () => {
        if (refItem === undefined || refItem.current === undefined || refItem.current === null) return

        if (refItem.current.paused) {
            setPlaying(true)
            refItem.current.play()
        } else {
            if (time && currentPodcastEpisode) {
                logCurrentPlaybackTime(currentPodcastEpisode.podcastEpisode.episode_id, time)
                const mappedEpisodes = selectedEpisodes.map(e=>{
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

            setPlaying(false)
            refItem.current?.pause()
        }
    }

    const changeSpeed = () => {
        if (refItem.current === null) return

        const currentIndex = SPEED_STEPS.indexOf(speed)

        if (currentIndex === SPEED_STEPS.length - 1) {
            refItem.current.playbackRate = SPEED_STEPS[0]!
            setPlaybackRate(SPEED_STEPS[0]!)
            return
        }

        refItem.current.playbackRate = SPEED_STEPS[currentIndex + 1]!
        setPlaybackRate(SPEED_STEPS[currentIndex + 1]!)
    }

    const seekForward = () => {
        if (refItem === undefined || refItem.current === undefined || refItem.current === null) return

        if (refItem.current.currentTime + SKIPPED_TIME < refItem.current.duration) {
            refItem.current.currentTime += SKIPPED_TIME
        }
    }

    const seekBackward = () => {
        if (refItem === undefined || refItem.current === undefined || refItem.current === null) return

        if (refItem.current.currentTime - SKIPPED_TIME > 0 ) {
            refItem.current.currentTime -= SKIPPED_TIME
        } else {
            refItem.current.currentTime = 0
        }
    }

    useKeyDown(seekBackward, ['j', 'ArrowLeft'], false)

    useKeyDown(seekForward, ['l', 'ArrowRight'], false)

    useKeyDown(handleButton, ['k', ' '], false)

    return (
        <div className="flex items-center justify-center gap-6">
            {/* Skip back */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90 " onClick={() => seekBackward()}>replay_30</span>

            {/* Previous */}
            <span className="material-symbols-outlined filled cursor-pointer text-3xl lg:text-4xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90" onClick={() => skipToPreviousEpisode()}>skip_previous</span>

            {/* Play/pause */}
            <span className="flex items-center justify-center bg-[--fg-color] hover:bg-[--fg-color-hover] cursor-pointer h-10 w-10 lg:h-12 lg:w-12 rounded-full active:scale-90" onClick={() => handleButton()}>
                {isPlaying?
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl text-[--bg-color]">pause</span>:
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl text-[--bg-color]">play_arrow</span>
                }
            </span>

            {/* Next */}
            <span className="material-symbols-outlined filled cursor-pointer text-3xl lg:text-4xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90" onClick={() => skipToNextEpisode()}>skip_next</span>

            {/* Skip forward */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90 " onClick={() => seekForward()}>forward_30</span>

            {/* Speed fixed width to prevent layout shift when value changes */}
            <span className="cursor-pointer text-sm text-[--fg-color] hover:text-[--fg-color-hover] w-8" onClick={() => changeSpeed()}>{speed}x</span>
        </div>
    )
}
