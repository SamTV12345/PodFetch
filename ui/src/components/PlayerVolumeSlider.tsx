import {FC, RefObject} from "react"
import * as Slider from "@radix-ui/react-slider"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setVolume} from "../store/AudioPlayerSlice"
import {AudioAmplifier} from "../models/AudioAmplifier"
import {VolumeIcon} from "../icons/VolumeIcon"

type PlayerVolumeSliderProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined
}

export const PlayerVolumeSlider:FC<PlayerVolumeSliderProps> = ({refItem, audioAmplifier}) => {
    const volume = useAppSelector(state=>state.audioPlayer.volume)
    const dispatch = useAppDispatch()

    return (
        <div className="flex items-center gap-2 w-40 sm:w-full sm:px-0">
            <VolumeIcon audio={refItem} volume={volume}/>

            {/*  Volume max 300 as some podcast providers have inconsistent sound profiles */}
            <Slider.Root className="relative flex items-center cursor-pointer h-2 w-full" value={[volume]} max={300} onValueChange={(v)=>{
                audioAmplifier&& audioAmplifier.setVolume(Number(v)/100)
                if(refItem && refItem.current){
                    dispatch(setVolume(Number(v)))

                }}
            }>
                <Slider.Track className="relative grow bg-stone-400 h-0.5">
                    <Slider.Range className="absolute bg-stone-900 h-full"/>
                </Slider.Track>

                <Slider.Thumb className="block bg-stone-900 h-2 w-2 rounded-full"/>
            </Slider.Root>

            {/* Fixed width to avoid layout shift when volume changes */}
            <span className="text-right text-xs w-10">{volume}%</span>
        </div>
    )
}
