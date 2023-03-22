import {FC, RefObject} from "react";
import {useAppSelector} from "../store/hooks";
import {HiddenAudioPlayer} from "./HiddenAudioPlayer";
import {AudioAmplifier} from "../models/AudioAmplifier";
import {VisualAudioPlayer} from "./VisualAudioPlayer";

type AudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined
    setAudioAmplifier: (audioAmplifier: AudioAmplifier|undefined)=>void
}
export const AudioPlayer:FC<AudioPlayerProps> = ({refItem, audioAmplifier, setAudioAmplifier}) => {
    const detailedAudioPodcastOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)


    return <div className="sticky bottom-0 w-full bg-gray-800 z-50" id="audio-bottom-bar">
        {!detailedAudioPodcastOpen&&<VisualAudioPlayer refItem={refItem} audioAmplifier={audioAmplifier}/>}
        <HiddenAudioPlayer refItem={refItem} setAudioAmplifier={setAudioAmplifier}/>
    </div>
}
