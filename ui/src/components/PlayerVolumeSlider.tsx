import { FC, RefObject } from 'react'
import * as Slider from '@radix-ui/react-slider'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { VolumeIcon } from '../icons/VolumeIcon'
import { useKeyDown } from '../hooks/useKeyDown'
import { VOLUME_STEP } from '../utils/Utilities';
import {getAudioPlayer} from "../utils/audioPlayer";

type PlayerVolumeSliderProps = {
    audioAmplifier: AudioAmplifier | undefined
}

export const PlayerVolumeSlider: FC<PlayerVolumeSliderProps> = ({ audioAmplifier }) => {
    const volume = useAudioPlayer(state => state.volume)
    const setVolume = useAudioPlayer(state => state.setVolume)

    useKeyDown(() => {
            const newVolume = Math.max(0, volume - VOLUME_STEP)
            setVolume(newVolume)
            const audioPlayer = getAudioPlayer()
            if (audioPlayer) {
                audioPlayer.volume = Math.min(1, newVolume / 100)
            }
            if (audioAmplifier) {
                audioAmplifier.setVolume(newVolume / 100)
            }
    }, ['ArrowDown'], false)

    useKeyDown(() => {
            const newVolume = Math.min(300, volume + VOLUME_STEP)
            setVolume(newVolume)
            const audioPlayer = getAudioPlayer()
            if (audioPlayer) {
                audioPlayer.volume = Math.min(1, newVolume / 100)
            }
            if (audioAmplifier) {
                audioAmplifier.setVolume(newVolume / 100)
            }
    }, ['ArrowUp'], false)

    const applyVolumeToPlayer = (newVolume: number) => {
        const audioPlayer = getAudioPlayer()
        if (audioPlayer) {
            audioPlayer.volume = Math.min(1, newVolume / 100)
        }
        if (audioAmplifier) {
            audioAmplifier.setVolume(newVolume / 100)
        }
    }

    return (
        <div className="flex items-center gap-2 w-40 sm:w-full sm:px-0">
            <VolumeIcon max={300} volume={volume}/>

            {/*  Volume max 300 as some podcast providers have inconsistent sound profiles */}
            <Slider.Root className="relative flex items-center cursor-pointer h-2 w-full" value={[volume]} max={300} onValueChange={(v) => {
                const newVolume = Number(v)
                setVolume(newVolume)
                applyVolumeToPlayer(newVolume)
            }}>
                <Slider.Track className="relative grow ui-slider-surface h-0.5">
                    <Slider.Range className="absolute ui-slider-fill h-full"/>
                </Slider.Track>

                <Slider.Thumb className="block ui-slider-fill h-2 w-2 rounded-full"/>
            </Slider.Root>

            {/* Fixed width to avoid layout shift when volume changes */}
            <span className="inline-block text-xs ui-text w-16">{volume}%</span>
        </div>
    )
}
