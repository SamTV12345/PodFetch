import { FC, useState } from 'react'
import { IconProps } from './PlayIcon'
import 'material-symbols/outlined.css'
import {getAudioPlayer} from "../utils/audioPlayer";

interface VolumeProps extends IconProps {
    max: number,
    volume: number
}

export const VolumeIcon: FC<VolumeProps> = ({ max, volume }) => {
    const [muted, setMuted] = useState(false)

    return muted ? (
        <span className="material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-(--fg-color) hover:text-(--fg-color-hover)" onClick={() => {
            const audioPlayer = getAudioPlayer()
            if (!audioPlayer) {
                return
            }
            audioPlayer.muted = false
            setMuted(false)
        }}>volume_off</span>
    ) : (
        <span className="material-symbols-outlined filled cursor-pointer text-xl lg:text-2xl text-(--fg-color) hover:text-(--fg-color-hover) inline-block" onClick={() => {
            const audioPlayer = getAudioPlayer()
            if (!audioPlayer) {
                return
            }
            audioPlayer.muted = true
            setMuted(true)
        }}>{(volume === 0) ? 'volume_mute' : ((volume / max) < 0.5) ? 'volume_down' : 'volume_up'}</span>
    )
}
