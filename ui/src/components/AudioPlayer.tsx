import {Activity, FC} from 'react'
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

    return <>
        <div id="media-player-parking-slot" className="hidden">
            <video
                id="audio-player"
                crossOrigin="anonymous"
                playsInline
                controls={false}
                className="hidden"
            />
        </div>
        <Activity mode={loadedPodcastEpisode ? "visible" : "hidden"}>
            <DrawerAudioPlayer audioAmplifier={audioAmplifier} />
            <HiddenAudioPlayer setAudioAmplifier={setAudioAmplifier} />
        </Activity>
    </>
}
