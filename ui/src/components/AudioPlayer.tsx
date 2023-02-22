import {PlayIcon} from "./PlayIcon";
import {createRef, useEffect} from "react";
import ProgressBar from "./AudioProgressBar";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentTimeUpdate, setMetadata, setPlaying} from "../store/AudioPlayerSlice";

export const AudioPlayer = () => {
    const dispatch = useAppDispatch()
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const podcast = useAppSelector(state=>state.audioPlayer.currentPodcast)

    const ref = createRef<HTMLAudioElement>()

    useEffect(()=>{
        if(podcastEpisode && playing){
            ref.current?.play()
        }
    },[podcastEpisode, playing])

    return <div className="sticky bottom-0 w-full bg-gray-800">
        <ProgressBar audioplayerRef={ref}/>
        <div className="grid grid-cols-3">
            <div className="grid place-items-start ml-5 mb-2">
                <div className="grid grid-cols-[auto_1fr] gap-2">
                    <img src={podcastEpisode?.image_url} alt="" className="w-10 h-10"/>
                    <div>
                        <div className="text-white">{podcastEpisode?.name}</div>
                        <div className="text-white text-sm">{podcast?.name}</div>
                    </div>
                </div>
            </div>
            <div className="grid place-items-center">
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
        <div></div>
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
            <source src={podcastEpisode?.url}/>
        </audio>
    </div>
}
