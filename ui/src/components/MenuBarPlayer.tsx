import {PlayIcon} from "./PlayIcon";
import {setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {FC, RefObject} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";

type MenuBarPlayerProps = {
    refItem: RefObject<HTMLAudioElement>
}
export const MenuBarPlayer:FC<MenuBarPlayerProps> = ({refItem}) => {
    const dispatch = useAppDispatch()
    const currentPodcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const episodes = useAppSelector(state=>state.common.selectedEpisodes)

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
        refItem.current.src = episodes[index].url
        refItem.current?.play()
        dispatch(setPlaying(true))
    }

    return  <div className="grid place-items-center">
        <div className="flex gap-5 align-baseline">
            <i className="fa-solid fa-backward text-xl text-white text-3xl" onClick={()=>skipToPreviousEpisode()}></i>
            <i className="fa-solid fa-play text-white align-top text-3xl" onClick={()=>{
                if(refItem===undefined || refItem.current===undefined|| refItem.current===null){
                    return
                }
                if(refItem.current.paused){
                    dispatch(setPlaying(true))
                    refItem.current.play()
                }else{
                    dispatch(setPlaying(false))
                    refItem.current?.pause()
                }
            }}/>
            <i className="fa-solid fa-forward h-6 text-xl text-white text-3xl" onClick={()=>{skipToNextEpisode()}}></i>
        </div>
    </div>
}
