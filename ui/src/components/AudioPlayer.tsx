import {PlayIcon} from "./PlayIcon";
import {createRef, useEffect} from "react";
import ProgressBar from "./AudioProgressBar";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentTimeUpdate, setMetadata, setPlaying, setVolume} from "../store/AudioPlayerSlice";
import {VolumeIcon} from "./VolumeIcon";
import {PreviewPlayer} from "./PreviewPlayer";
import {MenuBarPlayer} from "./MenuBarPlayer";
import {HiddenAudioPlayer} from "./HiddenAudioPlayer";

export const AudioPlayer = () => {
    const dispatch = useAppDispatch()
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const volume = useAppSelector(state=>state.audioPlayer.volume)
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
            <PreviewPlayer podcast={podcast} podcastEpisode={podcastEpisode}/>
            <MenuBarPlayer refItem={ref}/>
        <div className="grid place-items-center">
            <div className="flex gap-3">
                <VolumeIcon className="text-white" audio={ref}/>
                <input type="range" value={volume} onChange={(e)=>{
                    if(ref && ref.current){
                        ref.current.volume = Number(e.currentTarget.value)/100
                        dispatch(setVolume(Number(e.currentTarget.value)))
                }}
                }/>
            </div>
        </div>
        </div>
        <HiddenAudioPlayer refItem={ref} podcastEpisode={podcastEpisode}/>
    </div>
}
