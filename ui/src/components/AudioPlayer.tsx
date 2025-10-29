import { Activity, type FC } from 'react'
import type { AudioAmplifier } from '../models/AudioAmplifier'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { DrawerAudioPlayer } from './DrawerAudioPlayer'
import { HiddenAudioPlayer } from './HiddenAudioPlayer'

type AudioPlayerProps = {
	audioAmplifier: AudioAmplifier | undefined
	setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const AudioPlayer: FC<AudioPlayerProps> = ({
	audioAmplifier,
	setAudioAmplifier,
}) => {
	const loadedPodcastEpisode = useAudioPlayer(
		(state) => state.loadedPodcastEpisode,
	)

	return (
		<>
			<Activity mode={loadedPodcastEpisode ? 'visible' : 'hidden'}>
				<DrawerAudioPlayer audioAmplifier={audioAmplifier} />
			</Activity>
			<HiddenAudioPlayer setAudioAmplifier={setAudioAmplifier} />
		</>
	)
}
