import {setCurrentTimeUpdate, setMetadata} from "../store/AudioPlayerSlice";
import {FC, RefObject} from "react";
import {useAppDispatch} from "../store/hooks";
import {PodcastEpisode} from "../store/CommonSlice";

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    podcastEpisode?: PodcastEpisode
}

export const HiddenAudioPlayer:FC<HiddenAudioPlayerProps> = ({refItem, podcastEpisode}) => {
    const dispatch = useAppDispatch()

    return <audio ref={refItem}  onTimeUpdate={(e)=>{
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
}
