import {AudioPlayer} from "./AudioPlayer";
import {DetailedAudioPlayer} from "./DetailedAudioPlayer";
import {createRef, useState} from "react";
import {useAppSelector} from "../store/hooks";
import {AudioAmplifier} from "../models/AudioAmplifier";

export const AudioComponents = () => {
    const ref = createRef<HTMLAudioElement>()
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    let  [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()
    const detailedAudioPodcastOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)
    return <>
        {currentPodcast && <AudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier}/>}
        {detailedAudioPodcastOpen && <DetailedAudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier}/>}
        </>
}
