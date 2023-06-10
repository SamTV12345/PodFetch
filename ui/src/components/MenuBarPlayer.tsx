import {setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {FC, RefObject} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {logCurrentPlaybackTime, SKIPPED_TIME} from "../utils/Utilities";

type MenuBarPlayerProps = {
    refItem: RefObject<HTMLAudioElement>
}
export const MenuBarPlayer:FC<MenuBarPlayerProps> = ({refItem}) => {
    const dispatch = useAppDispatch()
    const currentPodcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const episodes = useAppSelector(state=>state.common.selectedEpisodes)
    const isPlaying  = useAppSelector(state=>state.audioPlayer.isPlaying)
    const time  = useAppSelector(state=>state.audioPlayer.metadata?.currentTime)
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

    return  <div className="grid place-items-center mr-5 md:mr-0 mb-1">
        <div className="flex gap-5 align-baseline">
            <div className="place-items-center grid">
                        <i className="text-2xl fa fa-clock-rotate-left text-white active:scale-75 hover:text-blue-500" onClick={()=>{
                            if (refItem.current===undefined||refItem.current===null){
                                return
                            }
                            if(refItem.current.currentTime-SKIPPED_TIME>0){
                                refItem.current.currentTime-=SKIPPED_TIME
                            }}}/>
            </div>
            <div className="place-items-center grid">
            <i className="fa-solid fa-backward text-xl text-white text-3xl hover:text-blue-500" onClick={()=>skipToPreviousEpisode()}></i>
            </div>
            <div className="relative rounded-full bg-black w-12 h-12 p-2 grid place-items-center bg-gray-900 hover:bg-gray-600 active:scale-75" onClick={()=>handleButton()}>
                {isPlaying?
                    <i className="fa-solid fa-pause text-3xl ml-1 text-white mr-1"></i>:
                    <i className=" fa-solid fa-play text-white align-top text-3xl ml-1"></i>
                }
            </div>
            <div className="grid place-items-center">
                <i className="fa-solid fa-forward h-6 text-xl text-white text-3xl hover:text-blue-500" onClick={()=>{skipToNextEpisode()}}></i>
            </div>
            <div className="place-items-center grid">
                <i className="text-2xl fa fa-clock-rotate-left fa-flip-horizontal  text-white active:scale-75 hover:text-blue-500" onClick={()=>{
                    if (refItem.current===undefined||refItem.current===null){
                        return
                    }
                    if(refItem.current.currentTime+SKIPPED_TIME<refItem.current.duration){
                   refItem.current.currentTime+=SKIPPED_TIME
                }}}/>
            </div>
        </div>
    </div>
}
