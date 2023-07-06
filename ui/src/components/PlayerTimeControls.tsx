import {FC, RefObject} from "react"
import {logCurrentPlaybackTime, SKIPPED_TIME} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setCurrentPodcastEpisode, setPlayBackRate, setPlaying} from "../store/AudioPlayerSlice"
import "material-symbols/outlined.css"

type PlayerTimeControlsProps = {
    refItem: RefObject<HTMLAudioElement>
}

export const PlayerTimeControls:FC<PlayerTimeControlsProps> = ({refItem}) => {
    const dispatch = useAppDispatch()
    const currentPodcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const episodes = useAppSelector(state=>state.common.selectedEpisodes)
    const isPlaying  = useAppSelector(state=>state.audioPlayer.isPlaying)
    const time  = useAppSelector(state=>state.audioPlayer.metadata?.currentTime)
    const speed = useAppSelector(state=>state.audioPlayer.playBackRate)

    const skipToPreviousEpisode = () => {
        if(currentPodcastEpisode===undefined){
            return
        }
        const index = episodes.findIndex(e=>e.id===currentPodcastEpisode.id)
        if(index===-1){
            return
        }
        if(index===0){
            return
        }
        switchToEpisodes(index-1)
    }

    const skipToNextEpisode = () => {
        if(currentPodcastEpisode===undefined){
            return
        }
        const index = episodes.findIndex(e=>e.id===currentPodcastEpisode.id)
        if(index===-1){
            return
        }
        if(index===episodes.length+1){
            return
        }
        switchToEpisodes(index+1)
    }

    const switchToEpisodes = (index: number)=>{
        if(refItem===undefined || refItem.current===undefined|| refItem.current===null){
            return
        }
        dispatch(setCurrentPodcastEpisode(episodes[index]))
        refItem.current.src = episodes[index].local_url
        refItem.current.load()
        refItem.current?.play()
        dispatch(setPlaying(true))
    }

    const handleButton = ()=>{
        if(refItem===undefined || refItem.current===undefined|| refItem.current===null){
            return
        }
        if(refItem.current.paused){
            dispatch(setPlaying(true))
            refItem.current.play()
        }else{
            if(time && currentPodcastEpisode){
                logCurrentPlaybackTime(currentPodcastEpisode.episode_id,time)
            }
            dispatch(setPlaying(false))
            refItem.current?.pause()
        }
    }

    const changeSpeed = ()=>{
        if (refItem.current===null) return
        let newSpeed = speed+0.5
        if (newSpeed>3) {
            newSpeed = 1
        }
        refItem.current.playbackRate = newSpeed
        dispatch(setPlayBackRate(newSpeed))
    }

    return (
        <div className="flex items-center justify-center gap-6">

            {/* Skip back */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-stone-900 hover:text-stone-600 active:scale-90 " onClick={()=>{
                if (refItem.current===undefined||refItem.current===null){
                    return
                }
                if(refItem.current.currentTime-SKIPPED_TIME>0){
                    refItem.current.currentTime-=SKIPPED_TIME
                }
            }}>replay_30</span>

            {/* Previous */}
            <span className="material-symbols-outlined filled cursor-pointer text-3xl lg:text-4xl text-stone-900 hover:text-stone-600 active:scale-90" onClick={()=>skipToPreviousEpisode()}>skip_previous</span>

            {/* Play/pause */}
            <span className="flex items-center justify-center bg-stone-900 hover:bg-stone-600 cursor-pointer h-10 w-10 lg:h-12 lg:w-12 rounded-full active:scale-90" onClick={()=>handleButton()}>
                {isPlaying?
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl text-white">pause</span>:
                    <span className="material-symbols-outlined filled text-2xl lg:text-4xl text-white">play_arrow</span>
                }
            </span>

            {/* Next */}
            <span className="material-symbols-outlined filled cursor-pointer text-3xl lg:text-4xl text-stone-900 hover:text-stone-600 active:scale-90" onClick={()=>skipToNextEpisode()}>skip_next</span>

            {/* Skip forward */}
            <span className="material-symbols-outlined cursor-pointer text-2xl lg:text-3xl text-stone-900 hover:text-stone-600 active:scale-90 " onClick={()=>{
                if (refItem.current===undefined||refItem.current===null){
                    return
                }
                if(refItem.current.currentTime+SKIPPED_TIME<refItem.current.duration){
                    refItem.current.currentTime+=SKIPPED_TIME
                }
            }}>forward_30</span>

            {/* Speed fixed width to prevent layout shift when value changes */}
            <span className="cursor-pointer text-sm text-stone-900 hover:text-stone-600 w-8" onClick={()=>changeSpeed()}>{speed}x</span>
        </div>
    )
}
