import { FC, RefObject } from 'react'
import { useAppSelector } from '../store/hooks'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { DrawerAudioPlayer } from './DrawerAudioPlayer'
import { HiddenAudioPlayer } from './HiddenAudioPlayer'

type AudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier | undefined
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const AudioPlayer: FC<AudioPlayerProps> = ({ refItem, audioAmplifier, setAudioAmplifier }) => {
    const detailedAudioPodcastOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)
    
    return <>
        {!detailedAudioPodcastOpen && <DrawerAudioPlayer refItem={refItem} audioAmplifier={audioAmplifier} />}
        <HiddenAudioPlayer refItem={refItem} setAudioAmplifier={setAudioAmplifier} />
    </>
}
