import { FC, RefObject, useState } from 'react'
import { IconProps } from './PlayIcon'
import 'material-symbols/outlined.css'

interface VolumeProps extends IconProps {
    audio:  RefObject<HTMLAudioElement>,
    max: number,
    volume: number
}

export const VolumeIcon: FC<VolumeProps> = ({ audio, max, volume }) => {
    const [muted, setMuted] = useState(false)

    return muted ? (
        <span className="material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-[--fg-color] hover:text-[--fg-color-hover]" onClick={() => {
            if(audio && audio.current) {
                audio.current.muted = false
                setMuted(false)
            }
        }}>volume_off</span>
    ) : (
        <span className="material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-[--fg-color] hover:text-[--fg-color-hover] inline-block" onClick={() => {
            if(audio && audio.current) {
                audio.current.muted = true
                setMuted(true)
            }
        }}>{(volume === 0) ? 'volume_mute' : ((volume / max) < 0.5) ? 'volume_down' : 'volume_up'}</span>
    )
}
