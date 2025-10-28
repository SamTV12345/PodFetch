import { Activity, type FC, RefObject } from 'react'
import type { AudioAmplifier } from '../models/AudioAmplifier'
import useAudioPlayer from '../store/AudioPlayerSlice'
import useCommon from '../store/CommonSlice'
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
		<Activity mode={loadedPodcastEpisode ? 'visible' : 'hidden'}>
			<DrawerAudioPlayer audioAmplifier={audioAmplifier} />
			<HiddenAudioPlayer setAudioAmplifier={setAudioAmplifier} />
		</Activity>
	)
}
