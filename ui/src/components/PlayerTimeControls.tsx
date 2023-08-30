import {FC, RefObject, useEffect} from 'react'
import {
    apiURL,
    logCurrentPlaybackTime,
    prepareOnlinePodcastEpisode,
    preparePodcastEpisode,
    SKIPPED_TIME
} from '../utils/Utilities'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { setCurrentPodcastEpisode, setPlayBackRate, setPlaying } from '../store/AudioPlayerSlice'
import 'material-symbols/outlined.css'
import {store} from "../store/store";
import axios, {AxiosResponse} from "axios";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {setSelectedEpisodes} from "../store/CommonSlice";
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";

type PlayerTimeControlsProps = {
    refItem: RefObject<HTMLAudioElement>
}

export const PlayerTimeControls: FC<PlayerTimeControlsProps> = ({ refItem }) => {
    const dispatch = useAppDispatch()
    const currentPodcastEpisode = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    const episodes = useAppSelector(state => state.common.selectedEpisodes)
    const isPlaying  = useAppSelector(state => state.audioPlayer.isPlaying)
    const speed = useAppSelector(state => state.audioPlayer.playBackRate)
    const time  = useAppSelector(state => state.audioPlayer.metadata?.currentTime)
    const selectedEpisodes = useAppSelector(state => state.common.selectedEpisodes)
    const skipToPreviousEpisode = () => {
        if (currentPodcastEpisode === undefined) return

        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode.id)

        if (index === -1) return

        if (index === 0) return

        switchToEpisodes(index - 1)
    }

    useEffect(() => {
        refItem.current!.onended = () => {
            logCurrentPlaybackTime(store.getState().audioPlayer.currentPodcastEpisode!.episode_id,
                store.getState().audioPlayer.currentPodcastEpisode!.total_time)
        }
    }, []);


    const skipToNextEpisode = () => {
        if (currentPodcastEpisode === undefined) return

        const index = episodes.findIndex(e => e.podcastEpisode.id === currentPodcastEpisode.id)

        if (index === -1) return

        if (index === episodes.length + 1) return

        switchToEpisodes(index + 1)
    }

    const switchToEpisodes = (index: number) => {
        if (refItem === undefined || refItem.current === undefined|| refItem.current === null) return

        const nextEpisode = episodes[index].podcastEpisode
        axios.get(apiURL + "/podcast/episode/" + nextEpisode.episode_id)
            .then((response: AxiosResponse<PodcastWatchedModel>) => {
                dispatch(setCurrentPodcastEpisode(nextEpisode))
                nextEpisode.status === 'D'
                    ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(nextEpisode, response.data)))
                    : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(nextEpisode, response.data)))
                refItem.current!.src = episodes[index].podcastEpisode.local_url
                refItem.current!.load()
                refItem.current?.play()
                dispatch(setPlaying(true))
            })

    }

    const handleButton = () => {
        if (refItem === undefined || refItem.current === undefined || refItem.current === null) return

        if (refItem.current.paused) {
            dispatch(setPlaying(true))
            refItem.current.play()
        } else {
            if (time && currentPodcastEpisode) {
                logCurrentPlaybackTime(currentPodcastEpisode.episode_id, time)
                const mappedEpisodes:EpisodesWithOptionalTimeline[] = selectedEpisodes.map(e=>{
                    if(e.podcastEpisode.episode_id === currentPodcastEpisode.episode_id){
                        return {
                            ...e,
                           podcastHistoryItem:{
                                 ...e.podcastHistoryItem!,
                               watchedTime: time
                           }
                        } satisfies EpisodesWithOptionalTimeline
                    }
                    return e
                })
                dispatch(setSelectedEpisodes(mappedEpisodes))
            }

            dispatch(setPlaying(false))
            refItem.current?.pause()
        }
    }

    const changeSpeed = () => {
        if (refItem.current === null) return

        let newSpeed = speed + 0.5

        if (newSpeed > 3) {
            newSpeed = 1
        }

        refItem.current.playbackRate = newSpeed
        dispatch(setPlayBackRate(newSpeed))
    }

    return (
        <div className="flex items-center justify-center gap-6">
            {/* Skip back */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90 " onClick={() => {
                if (refItem.current === undefined || refItem.current === null) return

                if (refItem.current.currentTime - SKIPPED_TIME > 0 ) {
                    refItem.current.currentTime -= SKIPPED_TIME
                }
            }}>replay_30</span>

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
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-[--fg-color] hover:text-[--fg-color-hover] active:scale-90 " onClick={() => {
                if (refItem.current === undefined || refItem.current === null) return

                if (refItem.current.currentTime + SKIPPED_TIME < refItem.current.duration) {
                    refItem.current.currentTime += SKIPPED_TIME
                }
            }}>forward_30</span>

            {/* Speed fixed width to prevent layout shift when value changes */}
            <span className="cursor-pointer text-sm text-[--fg-color] hover:text-[--fg-color-hover] w-8" onClick={() => changeSpeed()}>{speed}x</span>
        </div>
    )
}
