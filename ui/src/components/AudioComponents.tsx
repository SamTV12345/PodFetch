import { createRef, useState } from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { AudioPlayer } from './AudioPlayer'
import { DetailedAudioPlayer } from './DetailedAudioPlayer'
import useAudioPlayer from "../store/AudioPlayerSlice";
import useCommon from "../store/CommonSlice";

export const AudioComponents = () => {
    const ref = createRef<HTMLAudioElement>()
    const currentPodcast = useAudioPlayer(state => state.currentPodcastEpisode)
    const detailedAudioPodcastOpen = useCommon(state => state.detailedAudioPlayerOpen)
    const [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()

    return (
        <>
            {currentPodcast && <AudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />}

            {detailedAudioPodcastOpen && <DetailedAudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />}
        </>
    )
}
