import { createRef, useState } from 'react'
import { useAppSelector } from '../store/hooks'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { AudioPlayer } from './AudioPlayer'
import { DetailedAudioPlayer } from './DetailedAudioPlayer'

export const AudioComponents = () => {
    const ref = createRef<HTMLAudioElement>()
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    const detailedAudioPodcastOpen = useAppSelector(state => state.common.detailedAudioPlayerOpen)
    const [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()

    return (
        <>
            {currentPodcast && <AudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />}

            {detailedAudioPodcastOpen && <DetailedAudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />}
        </>
    )
}
