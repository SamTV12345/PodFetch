import {Activity, FC, RefObject} from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { DrawerAudioPlayer } from './DrawerAudioPlayer'
import { HiddenAudioPlayer } from './HiddenAudioPlayer'
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";

type AudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement|null>,
    audioAmplifier: AudioAmplifier | undefined
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const AudioPlayer: FC<AudioPlayerProps> = ({ refItem, audioAmplifier, setAudioAmplifier }) => {
    const loadedPodcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)

    return <Activity mode={loadedPodcastEpisode ? "visible" : "hidden"}>
        <DrawerAudioPlayer refItem={refItem} audioAmplifier={audioAmplifier} />
        <HiddenAudioPlayer refItem={refItem} setAudioAmplifier={setAudioAmplifier} />
    </Activity>
}
