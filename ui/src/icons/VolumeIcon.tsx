import { FC, useState } from 'react'
import { IconProps } from './PlayIcon'
import { Volume, Volume1, Volume2, VolumeX } from 'lucide-react'
import {getAudioPlayer} from "../utils/audioPlayer";

interface VolumeProps extends IconProps {
    max: number,
    volume: number
}

const ICON_CLASS = "cursor-pointer ui-text hover:ui-text-hover"
const ICON_SIZE = 22

export const VolumeIcon: FC<VolumeProps> = ({ max, volume }) => {
    const [muted, setMuted] = useState(false)

    const handleClick = (next: boolean) => {
        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) return
        audioPlayer.muted = next
        setMuted(next)
    }

    if (muted) {
        return <VolumeX size={ICON_SIZE} className={ICON_CLASS} onClick={() => handleClick(false)} />
    }
    if (volume === 0) {
        return <Volume size={ICON_SIZE} className={ICON_CLASS} onClick={() => handleClick(true)} />
    }
    if ((volume / max) < 0.5) {
        return <Volume1 size={ICON_SIZE} className={ICON_CLASS} onClick={() => handleClick(true)} />
    }
    return <Volume2 size={ICON_SIZE} className={ICON_CLASS} onClick={() => handleClick(true)} />
}
