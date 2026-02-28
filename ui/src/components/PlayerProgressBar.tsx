import React, {FC, useEffect, useMemo, useRef, useState} from 'react'
import useAudioPlayer, {type AudioPlayerPlay} from '../store/AudioPlayerSlice'
import {getAudioPlayer} from "../utils/audioPlayer";
import {usePlaybackLogger} from "../hooks/usePlaybackLogger";

type PlayerProgressBarProps = {
    className?: string,
    currentPodcastEpisode?: AudioPlayerPlay
}

const convertToMinutes = (time: number | undefined) => {
    if (time === undefined) {
        return '00:00:00'
    }

    const timeToConvert = Number(time?.toFixed(0))
    let hours = Math.floor(timeToConvert / 3600)

    let minutes = Math.floor(timeToConvert / 60) % 60
    let seconds = timeToConvert % 60
    let minutes_p = String(minutes).padStart(2, '0')
    let hours_p = String(hours).padStart(2, '0')
    let seconds_p = String(seconds).padStart(2, '0')

    if (hours_p === '00') {
        return minutes_p + ':' + seconds_p.substring(0, 2)
    }

    return hours_p + ':' + minutes_p + ':' + seconds_p.substring(0,2)
}

export const PlayerProgressBar: FC<PlayerProgressBarProps> = ({ className, currentPodcastEpisode }) => {
    const logCurrentPlaybackTime = usePlaybackLogger()
    const wrapper = useRef<HTMLDivElement>(null)
    const metadata = useAudioPlayer(state => state.metadata)
    const minute = useAudioPlayer(state => state.metadata?.currentTime)
    const [isDragging, setIsDragging] = useState(false)
    const [dragPercentage, setDragPercentage] = useState<number | null>(null)
    const setCurrentTimeUpdatePercentage = useAudioPlayer(state => state.setCurrentTimeUpdatePercentage)

    const totalDuration = useMemo(() => {
        return convertToMinutes(metadata?.duration)
    }, [metadata?.duration])

    const currentTime = useMemo(() => {
        return convertToMinutes(minute)
    }, [minute])

    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        e.preventDefault()
        e.stopPropagation()
        setIsDragging(true)

        if (wrapper.current) {
            const offset = wrapper.current.getBoundingClientRect()
            const localX = e.clientX - offset.left
            const percentage = Math.max(0, Math.min(100, (localX / offset.width) * 100))
            setDragPercentage(percentage)
        }
    }

    const handleWrapperClick = (e: React.MouseEvent<HTMLDivElement>) => {
        if (isDragging || !wrapper.current || !metadata) return

        e.preventDefault()
        e.stopPropagation()

        const offset = wrapper.current.getBoundingClientRect()
        const localX = e.clientX - offset.left
        const percentage = Math.max(0, Math.min(100, (localX / offset.width) * 100))

        const audioPlayer = getAudioPlayer()
        if (!audioPlayer) {
            return
        }
        const knownDuration = Number.isFinite(audioPlayer.duration) && audioPlayer.duration > 0
            ? audioPlayer.duration
            : metadata.duration
        const newTime = Math.floor((percentage / 100) * knownDuration)
        audioPlayer.currentTime = newTime
        setCurrentTimeUpdatePercentage(percentage)

        if (currentPodcastEpisode) {
            logCurrentPlaybackTime(currentPodcastEpisode.podcastEpisode.episode_id, newTime)
        }
    }

    const rafRef = React.useRef<number | null>(null)

    const handleMouseMove = (e: MouseEvent) => {
        if (!isDragging || !wrapper.current) return

        if (rafRef.current) {
            cancelAnimationFrame(rafRef.current)
        }

        rafRef.current = requestAnimationFrame(() => {
            if (!wrapper.current) return
            const offset = wrapper.current.getBoundingClientRect()
            const localX = e.clientX - offset.left
            const percentage = Math.max(0, Math.min(100, (localX / offset.width) * 100))
            setDragPercentage(percentage)
        })
    }

    const handleMouseUp = () => {
        if (isDragging && dragPercentage !== null && metadata) {
            const audioPlayer = getAudioPlayer()
            if (!audioPlayer) {
                return
            }
            const knownDuration = Number.isFinite(audioPlayer.duration) && audioPlayer.duration > 0
                ? audioPlayer.duration
                : metadata.duration
            const newTime = Math.floor((dragPercentage / 100) * knownDuration)
            audioPlayer.currentTime = newTime
            setCurrentTimeUpdatePercentage(dragPercentage)

            if (currentPodcastEpisode) {
                logCurrentPlaybackTime(currentPodcastEpisode.podcastEpisode.episode_id, newTime)
            }
        }
        setIsDragging(false)
        setDragPercentage(null)
    }

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
    }, [isDragging, dragPercentage, currentPodcastEpisode, metadata])

    const displayPercentage = Math.max(0, Math.min(100, isDragging && dragPercentage !== null ? dragPercentage : (metadata?.percentage ?? 0)))

    return (
        <div aria-controls="playbar" className="flex items-center gap-3">
            <span className={`text-xs text-right ui-text w-12 ${className}`}>{currentTime}</span>

            <div
                className="grow ui-slider-surface cursor-pointer h-1"
                ref={wrapper}
                onClick={handleWrapperClick}
                onMouseDown={handleMouseDown}
            >
                <div className="relative ui-slider-fill h-1 text-right" style={{width: displayPercentage + '%'}}>
                    <span
                        className="absolute -right-1 -top-1 ui-slider-fill h-3 w-3 rounded-full cursor-grab active:cursor-grabbing">
                    </span>
                </div>
            </div>

            <div className={`text-xs ui-text w-12 ${className}`}>{totalDuration}</div>
        </div>
    )
}
