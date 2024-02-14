import React, { createRef, FC, useMemo, useState } from 'react'
import useAudioPlayer from '../store/AudioPlayerSlice'
import {logCurrentPlaybackTime} from "../utils/navigationUtils";

type PlayerProgressBarProps = {
    audioplayerRef: React.RefObject<HTMLAudioElement>,
    className?: string
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

export const PlayerProgressBar: FC<PlayerProgressBarProps> = ({ audioplayerRef, className }) => {
    window.addEventListener('mousedown', () => {
        setMousePressed(true)
    })

    window.addEventListener('mouseup', () => {
        setMousePressed(false)
    })

    const control = createRef<HTMLElement>()
    const wrapper = createRef<HTMLDivElement>()
    const currentPodcastEpisode = useAudioPlayer(state => state.currentPodcastEpisode)
    const metadata = useAudioPlayer(state => state.metadata)
    const minute = useAudioPlayer(state => state.metadata?.currentTime)
    const time = useAudioPlayer(state => state.metadata?.currentTime)
    const [mousePressed, setMousePressed] = useState(false);
    const setCurrentTimeUpdatePercentage = useAudioPlayer(state => state.setCurrentTimeUpdatePercentage)

    const totalDuration = useMemo(() => {
        return convertToMinutes(metadata?.duration)
    }, [metadata?.duration])

    const currentTime = useMemo(() => {
        return convertToMinutes(minute)
    }, [minute])

    if (audioplayerRef === undefined || audioplayerRef.current === undefined || metadata === undefined) {
        return <div>test</div>
    }

    const endWrapperPosition = (e: React.MouseEvent<HTMLDivElement>) => {
        const offset = wrapper.current?.getBoundingClientRect()

        if (offset) {
            const localX = e.clientX - offset.left
            const percentage = localX / offset.width * 100

            if (percentage && audioplayerRef.current) {
                audioplayerRef.current.currentTime = Math.floor(percentage / 100 * audioplayerRef.current.duration)

                if (time && currentPodcastEpisode) {
                    logCurrentPlaybackTime(currentPodcastEpisode.episode_id, Number(audioplayerRef.current.currentTime.toFixed(0)))
                }
            }
        }
    }

    const calcTotalMovement = (e: React.MouseEvent<HTMLElement, MouseEvent>) => {
        if (mousePressed && metadata && audioplayerRef.current) {
            setCurrentTimeUpdatePercentage(metadata.percentage + e.movementX)
            audioplayerRef.current.currentTime = Math.floor(metadata.percentage + e.movementX / 100 * audioplayerRef.current.duration)
        }
    }

    return (
        <div className="flex items-center gap-3">
            {/* Fixed width to avoid layout shift as time progresses */}
            <span className={`text-xs text-right text-[--fg-color] w-12 ${className}`}>{currentTime}</span>

            <div className="grow bg-[--slider-bg-color] cursor-pointer h-1" ref={wrapper} onClick={(e) => {
                endWrapperPosition(e)
            }}>
                <div className="relative bg-[--slider-fg-color] h-1 text-right" style={{width: (metadata.percentage) + '%'}}>
                    <span className="absolute -right-1 -top-1 bg-[--slider-fg-color] h-3 w-3 rounded-full" onMouseMove={(e) => calcTotalMovement(e)} ref={control}></span>
                </div>
            </div>

            <div className={`text-xs text-[--fg-color] w-12 ${className}`}>{totalDuration}</div>
        </div>
    )
}
