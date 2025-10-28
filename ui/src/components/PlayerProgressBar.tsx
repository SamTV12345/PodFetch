import React, {
	type FC,
	useCallback,
	useEffect,
	useMemo,
	useState,
} from 'react'
import useAudioPlayer, { type AudioPlayerPlay } from '../store/AudioPlayerSlice'
import { getAudioPlayer } from '../utils/audioPlayer'
import { logCurrentPlaybackTime } from '../utils/navigationUtils'

type PlayerProgressBarProps = {
	className?: string
	currentPodcastEpisode?: AudioPlayerPlay
}

const convertToMinutes = (time: number | undefined) => {
	if (time === undefined) {
		return '00:00:00'
	}

	const timeToConvert = Number(time?.toFixed(0))
	const hours = Math.floor(timeToConvert / 3600)

	const minutes = Math.floor(timeToConvert / 60) % 60
	const seconds = timeToConvert % 60
	const minutes_p = String(minutes).padStart(2, '0')
	const hours_p = String(hours).padStart(2, '0')
	const seconds_p = String(seconds).padStart(2, '0')

	if (hours_p === '00') {
		return `${minutes_p}:${seconds_p.substring(0, 2)}`
	}

	return `${hours_p}:${minutes_p}:${seconds_p.substring(0, 2)}`
}

export const PlayerProgressBar: FC<PlayerProgressBarProps> = ({
	className,
	currentPodcastEpisode,
}) => {
	const wrapperRef = useRef<HTMLDivElement | null>(null)
	const metadata = useAudioPlayer((state) => state.metadata)
	const minute = useAudioPlayer((state) => state.metadata?.currentTime)
	const [isDragging, setIsDragging] = useState(false)
	const [dragPercentage, setDragPercentage] = useState<number | null>(null)
	const setCurrentTimeUpdatePercentage = useAudioPlayer(
		(state) => state.setCurrentTimeUpdatePercentage,
	)

	const totalDuration = useMemo(() => {
		return convertToMinutes(metadata?.duration)
	}, [metadata?.duration])

	const currentTime = useMemo(() => {
		return convertToMinutes(minute)
	}, [minute])

	const handleMouseUp = useCallback(
		(_e?: MouseEvent) => {
			if (isDragging && dragPercentage !== null && metadata) {
				const audioPlayer = getAudioPlayer()
				const newTime = Math.floor(
					(dragPercentage / 100) * audioPlayer.duration,
				)
				audioPlayer.currentTime = newTime
				setCurrentTimeUpdatePercentage(dragPercentage)

				if (currentPodcastEpisode) {
					logCurrentPlaybackTime(
						currentPodcastEpisode.podcastEpisode.episode_id,
						newTime,
					)
				}
			}
			setIsDragging(false)
			setDragPercentage(null)
		},
		[
			isDragging,
			dragPercentage,
			metadata,
			currentPodcastEpisode,
			setCurrentTimeUpdatePercentage,
		],
	)

	const handleMouseDown = useCallback(
		(e: React.MouseEvent<HTMLDivElement>) => {
			e.preventDefault()
			e.stopPropagation()
			setIsDragging(true)

			if (wrapperRef.current) {
				const offset = wrapperRef.current.getBoundingClientRect()
				const localX = e.clientX - offset.left
				const percentage = Math.max(
					0,
					Math.min(100, (localX / offset.width) * 100),
				)
				setDragPercentage(percentage)
			}
		},
		[wrapperRef.current],
	)

	const handleWrapperClick = useCallback(
		(e: React.MouseEvent<HTMLDivElement>) => {
			if (isDragging || !wrapperRef.current || !metadata) return

			e.preventDefault()
			e.stopPropagation()

			const offset = wrapperRef.current.getBoundingClientRect()
			const localX = e.clientX - offset.left
			const percentage = Math.max(
				0,
				Math.min(100, (localX / offset.width) * 100),
			)

			const audioPlayer = getAudioPlayer()
			const newTime = Math.floor((percentage / 100) * audioPlayer.duration)
			audioPlayer.currentTime = newTime
			setCurrentTimeUpdatePercentage(percentage)

			if (currentPodcastEpisode) {
				logCurrentPlaybackTime(
					currentPodcastEpisode.podcastEpisode.episode_id,
					newTime,
				)
			}
		},
		[
			isDragging,
			metadata,
			currentPodcastEpisode,
			setCurrentTimeUpdatePercentage,
			wrapperRef.current,
		],
	)

	const rafRef = React.useRef<number | null>(null)

	const handleMouseMove = useCallback(
		(e: MouseEvent) => {
			if (!isDragging || !wrapperRef.current) return

			if (rafRef.current) {
				cancelAnimationFrame(rafRef.current)
			}

			rafRef.current = requestAnimationFrame(() => {
				if (!wrapperRef.current) return
				const offset = wrapperRef.current.getBoundingClientRect()
				const localX = e.clientX - offset.left
				const percentage = Math.max(
					0,
					Math.min(100, (localX / offset.width) * 100),
				)
				setDragPercentage(percentage)
			})
		},
		[isDragging, wrapperRef.current],
	)

	useEffect(() => {
		if (isDragging) {
			document.addEventListener('mousemove', handleMouseMove, { passive: true })
			document.addEventListener('mouseup', handleMouseUp)

			return () => {
				document.removeEventListener('mousemove', handleMouseMove)
				document.removeEventListener('mouseup', handleMouseUp)
				if (rafRef.current) {
					cancelAnimationFrame(rafRef.current)
				}
			}
		}
	}, [isDragging, handleMouseMove, handleMouseUp])

	const displayPercentage =
		isDragging && dragPercentage !== null
			? dragPercentage
			: metadata?.percentage

	return (
		<div aria-controls="playbar" className="flex items-center gap-3">
			<span
				className={`text-xs text-right text-(--fg-color) w-12 ${className}`}
			>
				{currentTime}
			</span>

			<div
				className="grow bg-(--slider-bg-color) cursor-pointer h-1"
				ref={wrapperRef}
				onClick={handleWrapperClick}
				onMouseDown={handleMouseDown}
			>
				<div
					className="relative bg-(--slider-fg-color) h-1 text-right"
					style={{ width: `${displayPercentage}%` }}
				>
					<span className="absolute -right-1 -top-1 bg-(--slider-fg-color) h-3 w-3 rounded-full cursor-grab active:cursor-grabbing"></span>
				</div>
			</div>

			<div className={`text-xs text-(--fg-color) w-12 ${className}`}>
				{totalDuration}
			</div>
		</div>
	)
}
