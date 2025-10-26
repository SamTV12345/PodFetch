import {Activity, FC, RefObject} from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { DrawerAudioPlayer } from './DrawerAudioPlayer'
import { HiddenAudioPlayer } from './HiddenAudioPlayer'
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";

type AudioPlayerProps = {
    audioAmplifier: AudioAmplifier | undefined
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const AudioPlayer: FC<AudioPlayerProps> = ({  audioAmplifier, setAudioAmplifier }) => {
    const loadedPodcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)

    return <Activity mode={loadedPodcastEpisode ? "visible" : "hidden"}>
        <DrawerAudioPlayer audioAmplifier={audioAmplifier} />
        <HiddenAudioPlayer setAudioAmplifier={setAudioAmplifier} />
    </Activity>
}
