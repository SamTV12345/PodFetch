import { FC, RefObject } from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { DrawerAudioPlayer } from './DrawerAudioPlayer'
import { HiddenAudioPlayer } from './HiddenAudioPlayer'
import useCommon from "../store/CommonSlice";

type AudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    audioAmplifier: AudioAmplifier | undefined
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const AudioPlayer: FC<AudioPlayerProps> = ({ refItem, audioAmplifier, setAudioAmplifier }) => {
    const detailedAudioPodcastOpen = useCommon(state => state.detailedAudioPlayerOpen)

    return <>
        {!detailedAudioPodcastOpen && <DrawerAudioPlayer refItem={refItem} audioAmplifier={audioAmplifier} />}
        <HiddenAudioPlayer refItem={refItem} setAudioAmplifier={setAudioAmplifier} />
    </>
}
