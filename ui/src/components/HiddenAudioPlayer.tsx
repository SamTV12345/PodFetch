import { type FC, useEffect } from 'react'
import useOnMount from '../hooks/useOnMount'
import { AudioAmplifier } from '../models/AudioAmplifier'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { getAudioPlayer } from '../utils/audioPlayer'

type HiddenAudioPlayerProps = {
	setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({
	setAudioAmplifier,
}) => {
	const _setMetadata = useAudioPlayer((state) => state.setMetadata)
	const _setCurrentTimeUpdate = useAudioPlayer(
		(state) => state.setCurrentTimeUpdate,
	)

	useEffect(() => {
		const audio = getAudioPlayer()

		const onPlaying = () => {
			useAudioPlayer.getState().setPlaying(true)
		}
		const onPause = () => {
			useAudioPlayer.getState().setPlaying(false)
		}

		audio.addEventListener('playing', onPlaying)
		audio.addEventListener('pause', onPause)

		return () => {
			audio.removeEventListener('playing', onPlaying)
			audio.removeEventListener('pause', onPause)
		}
	}, [])

	useOnMount(() => {
		if (
			/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
				navigator.userAgent,
			)
		) {
			return
		}

		const audioPlayer = document.getElementById(
			'audio-player',
		) as HTMLAudioElement

		setAudioAmplifier(new AudioAmplifier(audioPlayer))
	})

	useEffect(() => {
		const audioPlayer = getAudioPlayer()
		const onTimeUpdate = (e: Event) => {
			const el = e.currentTarget as HTMLMediaElement
			if (useAudioPlayer.getState().metadata === undefined) {
				useAudioPlayer.getState().setMetadata({
					currentTime: el.currentTime,
					duration: el.duration,
					percentage: 0,
				})
			} else {
				useAudioPlayer.getState().setCurrentTimeUpdate(el.currentTime)
			}
		}
		audioPlayer.addEventListener('timeupdate', onTimeUpdate)

		return () => {
			audioPlayer.removeEventListener('timeupdate', onTimeUpdate)
		}
	}, [])

	return <div></div>
}
