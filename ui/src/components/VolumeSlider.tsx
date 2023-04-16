import {VolumeIcon} from "./VolumeIcon";
import {setPlayBackRate, setVolume} from "../store/AudioPlayerSlice";
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
    const speed = useAppSelector(state=>state.audioPlayer.playBackRate)

    const changeSpeed = ()=>{
        if (refItem.current===null) return
        let newSpeed = speed+0.5
        if (newSpeed>3) {
            newSpeed = 1
        }
        refItem.current.playbackRate = newSpeed
        dispatch(setPlayBackRate(newSpeed))
    }

    return         <div className="hidden md:grid place-items-center">
        <div className="flex gap-3">
            <div className="text-white mr-5 cursor-pointer" onClick={()=>changeSpeed()}>{speed}x</div>
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
