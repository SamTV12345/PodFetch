import {FC, RefObject, useEffect} from "react"
import {useAppSelector} from "../store/hooks"
import {AudioAmplifier} from "../models/AudioAmplifier"
import {PlayerTimeControls} from "./PlayerTimeControls"
import {PlayerEpisodeInfo} from "./PlayerEpisodeInfo"
import {PlayerProgressBar} from "./PlayerProgressBar"
import {PlayerVolumeSlider} from "./PlayerVolumeSlider"

type DrawerAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined
}

export const DrawerAudioPlayer: FC<DrawerAudioPlayerProps> = ({refItem, audioAmplifier}) => {
    const podcast = useAppSelector(state=>state.audioPlayer.currentPodcast)
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)

    useEffect(()=>{
        if(podcastEpisode && playing){
            refItem.current?.play()
        }
    },[podcastEpisode, playing])

    return (
        <div className="
            col-span-2 grid
            sm:grid-cols-[10rem_1fr_8rem] md:grid-cols-[16rem_1fr_10rem] lg:grid-cols-[20rem_1fr_12rem]
            justify-items-center sm:justify-items-stretch
            gap-2 sm:gap-4 lg:gap-10
            p-4 lg:pr-8
            bg-white shadow-[0_-4px_16px_rgba(0,0,0,0.15)]
        ">
            <PlayerEpisodeInfo podcast={podcast} podcastEpisode={podcastEpisode}/>

            <div className="flex flex-col gap-2 w-full">
                <PlayerTimeControls refItem={refItem}/>
                <PlayerProgressBar audioplayerRef={refItem}/>
            </div>

            <PlayerVolumeSlider audioAmplifier={audioAmplifier} refItem={refItem}/>
        </div>
    )
}
