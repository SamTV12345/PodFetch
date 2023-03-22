import {FC, RefObject, useEffect} from "react";
import ProgressBar from "./AudioProgressBar";
import {PreviewPlayer} from "./PreviewPlayer";
import {MenuBarPlayer} from "./MenuBarPlayer";
import {useAppSelector} from "../store/hooks";
import {AudioAmplifier} from "../models/AudioAmplifier";
import {VolumeSlider} from "./VolumeSlider";

type VisualAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined
}

export const VisualAudioPlayer: FC<VisualAudioPlayerProps> = ({refItem, audioAmplifier}) => {
    const podcast = useAppSelector(state=>state.audioPlayer.currentPodcast)
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)



    useEffect(()=>{
        if(podcastEpisode && playing){
            refItem.current?.play()
        }
    },[podcastEpisode, playing])



    return <>
        <ProgressBar audioplayerRef={refItem}/>
    <div className="grid md:grid-cols-3 grid-cols-[1fr_auto]">
        <PreviewPlayer podcast={podcast} podcastEpisode={podcastEpisode}/>
        <MenuBarPlayer refItem={refItem}/>
        <VolumeSlider audioAmplifier={audioAmplifier} refItem={refItem}/>
    </div>
        </>
}
