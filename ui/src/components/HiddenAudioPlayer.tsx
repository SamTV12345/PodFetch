import { type FC, RefObject, SyntheticEvent, useEffect } from 'react'
import useOnMount from '../hooks/useOnMount'
import { AudioAmplifier } from '../models/AudioAmplifier'
import useAudioPlayer from '../store/AudioPlayerSlice'
import { getAudioPlayer } from '../utils/audioPlayer'
import { client } from '../utils/http'

type HiddenAudioPlayerProps = {
	setAudioAmplifier: (audioAmplifier: AudioAmplifier) => void
}

export const HiddenAudioPlayer: FC<HiddenAudioPlayerProps> = ({
	setAudioAmplifier,
}) => {
	const podcastEpisode = useAudioPlayer((state) => state.loadedPodcastEpisode)
	const setMetadata = useAudioPlayer((state) => state.setMetadata)
	const setCurrentTimeUpdate = useAudioPlayer(
		(state) => state.setCurrentTimeUpdate,
	)

	const setPlaying = useAudioPlayer((state) => state.setPlaying)

	useEffect(() => {
		const audio = getAudioPlayer()
		if (!audio) return

		const onPlaying = () => {
			setPlaying(true)
		}
		const onPause = () => {
			setPlaying(false)
		}

		audio.addEventListener('playing', onPlaying)
		audio.addEventListener('pause', onPause)

		return () => {
			audio.removeEventListener('playing', onPlaying)
			audio.removeEventListener('pause', onPause)
		}
	}, [setPlaying])

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
		const audioPlayer = document.getElementById(
			'audio-player',
		) as HTMLAudioElement
		const onTimeUpdate = (e: Event) => {
			const el = e.currentTarget as HTMLMediaElement
			setCurrentTimeUpdate(el.currentTime)
		}
		const onLoadedMetadata = (e: Event) => {
			const el = e.currentTarget as HTMLMediaElement
			setMetadata({
				currentTime: el.currentTime,
				duration: el.duration,
				percentage: 0,
			})

			if (isNaN(el.duration)) {
				client
					.GET('/api/v1/podcasts/episode/{id}', {
						params: { path: { id: podcastEpisode!.podcastEpisode.episode_id } },
					})
					.then((response) => {
						setMetadata({
							currentTime: el.currentTime,
							duration: response.data!.total!,
							percentage: 0,
						})
					})
			}
		}

		audioPlayer.addEventListener('timeupdate', onTimeUpdate)
		audioPlayer.addEventListener('loadedmetadata', onLoadedMetadata)

		return () => {
			audioPlayer.removeEventListener('timeupdate', onTimeUpdate)
			audioPlayer.removeEventListener('loadedmetadata', onLoadedMetadata)
		}
	}, [])

	return <div></div>
}
