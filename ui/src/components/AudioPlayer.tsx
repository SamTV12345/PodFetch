import {createRef, useEffect, useState} from "react";
import ProgressBar from "./AudioProgressBar";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import { setVolume} from "../store/AudioPlayerSlice";
import {VolumeIcon} from "./VolumeIcon";
import {PreviewPlayer} from "./PreviewPlayer";
import {MenuBarPlayer} from "./MenuBarPlayer";
import {HiddenAudioPlayer} from "./HiddenAudioPlayer";
import {AudioAmplifier} from "../models/AudioAmplifier";
import useOnMount from "../hooks/useOnMount";

const amplifyMedia = (mediaElem: HTMLAudioElement, multiplier:number)=> {
    const context = new (window.AudioContext),
        result = {
            context: context,
            source: context.createMediaElementSource(mediaElem),
            gain: context.createGain(),
            media: mediaElem,
            amplify: (multiplier: number)=> { result.gain.gain.value = multiplier; },
            getAmpLevel: function() { return result.gain.gain.value; }
        };
    result.source.connect(result.gain);
    result.gain.connect(context.destination);
    result.amplify(multiplier);
    return result;
}

export const AudioPlayer = () => {
    const dispatch = useAppDispatch()
    const playing = useAppSelector(state=>state.audioPlayer.isPlaying)
    const volume = useAppSelector(state=>state.audioPlayer.volume)
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)
    const podcast = useAppSelector(state=>state.audioPlayer.currentPodcast)
    const ref = createRef<HTMLAudioElement>()
    let  [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()

    useEffect(()=>{
        if(podcastEpisode && playing){
            ref.current?.play()
        }
    },[podcastEpisode, playing])

    useOnMount(()=>{
            setAudioAmplifier(new AudioAmplifier(ref.current!))
    })

    return <div className="sticky bottom-0 w-full bg-gray-800 z-50" id="audio-bottom-bar">
        <ProgressBar audioplayerRef={ref}/>
        <div className="grid md:grid-cols-3 grid-cols-[1fr_auto]">
            <PreviewPlayer podcast={podcast} podcastEpisode={podcastEpisode}/>
            <MenuBarPlayer refItem={ref}/>
            <div className="hidden md:grid place-items-center">
                <div className="flex gap-3">
                    <VolumeIcon className="text-white hover:text-blue-500" audio={ref}/>
                    <input type="range" value={volume} max={300} onChange={(e)=>{
                        audioAmplifier&& audioAmplifier.setVolume(Number(e.currentTarget.value)/100)
                        if(ref && ref.current){
                            dispatch(setVolume(Number(e.currentTarget.value)))

                    }}
                    }/>
                    <span className="text-white">{volume}%</span>
                </div>
        </div>
        </div>
        <HiddenAudioPlayer refItem={ref}/>
    </div>
}
