import {VolumeIcon} from "./VolumeIcon";
import {setVolume} from "../store/AudioPlayerSlice";
import {AudioAmplifier} from "../models/AudioAmplifier";
import {FC, RefObject} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";

type VolumeSliderProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier|undefined
}

export const VolumeSlider:FC<VolumeSliderProps> = ({refItem, audioAmplifier}) => {
    const volume = useAppSelector(state=>state.audioPlayer.volume)
    const dispatch = useAppDispatch()

    return         <div className="hidden md:grid place-items-center">
        <div className="flex gap-3">
            <VolumeIcon className="text-white hover:text-blue-500" audio={refItem}/>
            <input type="range" value={volume} max={300} onChange={(e)=>{
                audioAmplifier&& audioAmplifier.setVolume(Number(e.currentTarget.value)/100)
                if(refItem && refItem.current){
                    dispatch(setVolume(Number(e.currentTarget.value)))

                }}
            }/>
            <span className="text-white">{volume}%</span>
        </div>
    </div>
}
