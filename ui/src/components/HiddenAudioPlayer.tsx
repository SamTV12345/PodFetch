import {setCurrentTimeUpdate, setMetadata} from "../store/AudioPlayerSlice";
import {FC, RefObject, useEffect} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>
}

export const HiddenAudioPlayer:FC<HiddenAudioPlayerProps> = ({refItem}) => {
    const dispatch = useAppDispatch()
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)

    useEffect(
        ()=>{
            if(podcastEpisode && refItem && refItem.current){
                console.log("Switched to episode: "+podcastEpisode.url+ " at time: "+podcastEpisode.time)
                refItem.current.load()
                refItem.current.currentTime = podcastEpisode.time
                refItem.current.play()
            }
        }
        ,[podcastEpisode])

    return <audio ref={refItem} src={podcastEpisode?.url} id={'hiddenaudio'} onTimeUpdate={(e)=>{
        dispatch(setCurrentTimeUpdate(e.currentTarget.currentTime))
    }} onLoadedMetadata={(e)=>{
        dispatch(setMetadata({
            currentTime: e.currentTarget.currentTime,
            duration: e.currentTarget.duration,
            percentage: 0
        }))
    }}/>
}
