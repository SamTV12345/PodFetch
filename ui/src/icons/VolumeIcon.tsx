import {FC, RefObject, useState} from "react"
import {IconProps} from "./PlayIcon"
import "material-symbols/outlined.css"

interface VolumeProps extends IconProps {
    audio:  RefObject<HTMLAudioElement>,
    className?: string,
    volume: number
}
export const VolumeIcon:FC<VolumeProps> = ({audio, className = '', volume}) => {
    const [muted, setMuted] = useState(false)

    return muted ? (
        <span className={`material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-stone-900 hover:text-stone-600 ${className}`} onClick={()=>{
            if(audio && audio.current) {
                audio.current.muted = false
                setMuted(false)
            }
        }}>volume_off</span>
    ) : (
        <span className={`material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-stone-900 hover:text-stone-600`} onClick={()=>{
            if(audio && audio.current) {
                audio.current.muted = true
                setMuted(true)
            }
        }}>{(volume < 50) ? 'volume_down' : 'volume_up'}</span>
    )
}
