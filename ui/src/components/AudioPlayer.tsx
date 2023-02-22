import {PlayIcon} from "./PlayIcon";
import {createRef, useEffect, useRef} from "react";
import ProgressBar from "./AudioProgressBar";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentTimeUpdate, setMetadata, setPlaying} from "../store/AudioPlayerSlice";
import {Simulate} from "react-dom/test-utils";
import playing = Simulate.playing;

export const AudioPlayer = () => {
    const dispatch = useAppDispatch()
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const podcast = useAppSelector(state=>state.audioPlayer.currentPodcast)
    const ref = createRef<HTMLAudioElement>()

    useEffect(()=>{
        if(podcast && playing){
            ref.current?.play()
        }
    },[podcast, playing])

    return <div className="sticky bottom-0 w-full bg-gray-800 h-12">
        <ProgressBar audioplayerRef={ref}/>
        <div className="place-items-center grid">
                <div className="flex gap-3 align-baseline">
                <i className="fa-solid fa-backward text-xl text-white"></i>
            <PlayIcon className="text-white align-top" onClick={()=>{
                if(ref.current?.paused){
                    dispatch(setPlaying(true))
                    ref.current.play()
                }else{
                    dispatch(setPlaying(false))
                    ref.current?.pause()
                }
            }}/>
                <i className="fa-solid fa-forward h-6 text-xl text-white"></i>
            </div>
        </div>
        <audio ref={ref}  onTimeUpdate={(e)=>{
            dispatch(setCurrentTimeUpdate(e.currentTarget.currentTime))
        }} onLoadedMetadata={(e)=>{
            dispatch(setMetadata({
                currentTime: e.currentTarget.currentTime,
                duration: e.currentTarget.duration,
                percentage: 0
            }))
        }}>
            <source src={podcast?.url}/>
        </audio>
    </div>
}
