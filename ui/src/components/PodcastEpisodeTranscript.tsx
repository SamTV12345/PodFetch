import {FC, useEffect, useMemo, useRef, useState} from 'react'
import {useTranslation} from 'react-i18next'
import {components} from '../../schema'
import {$api} from '../utils/http'
import {startAudioPlayer} from '../utils/audioPlayer'
import useAudioPlayer from '../store/AudioPlayerSlice'

type PodcastEpisodeTranscriptProps = {
    podcastEpisode: components["schemas"]["PodcastEpisodeDto"],
    className?: string
}

const formatTimestamp = (ms: number) => {
    const totalSeconds = Math.floor(ms / 1000)
    const minutes = Math.floor(totalSeconds / 60)
    const seconds = totalSeconds % 60
    return `${minutes}:${seconds.toString().padStart(2, '0')}`
}

export const PodcastEpisodeTranscript: FC<PodcastEpisodeTranscriptProps> = ({podcastEpisode, className}) => {
    const {t} = useTranslation()
    const [autoScroll, setAutoScroll] = useState(true)
    const currentTime = useAudioPlayer(state => state.metadata?.currentTime)
    const activeSegmentRef = useRef<HTMLLIElement | null>(null)

    const transcript = $api.useQuery('get', '/api/v1/podcasts/episodes/{id}/transcript', {
        params: {
            path: {
                id: podcastEpisode.id
            }
        }
    }, {
        // A 404 simply means the episode has no transcript — don't retry.
        retry: false
    })

    const segments = useMemo(() => transcript.data?.segments ?? [], [transcript.data])
    const hasTimestamps = useMemo(() => segments.some(segment => segment.startMs != null), [segments])

    const activeSegmentIdx = useMemo(() => {
        if (!hasTimestamps || currentTime === undefined) {
            return undefined
        }
        const currentMs = currentTime * 1000
        let active: number | undefined
        for (const segment of segments) {
            if (segment.startMs != null && segment.startMs <= currentMs) {
                if (segment.endMs == null || currentMs < segment.endMs || active === undefined) {
                    active = segment.idx
                }
            }
        }
        return active
    }, [segments, hasTimestamps, currentTime])

    useEffect(() => {
        if (autoScroll && activeSegmentRef.current) {
            activeSegmentRef.current.scrollIntoView({block: 'nearest'})
        }
    }, [activeSegmentIdx, autoScroll])

    if (transcript.isLoading) {
        return null
    }

    if (transcript.isError || segments.length === 0) {
        return <p className={`ui-text-muted ${className ?? ''}`}>{t('no-transcript-available')}</p>
    }

    return <div className={className}>
        {hasTimestamps && (
            <label className="flex items-center gap-2 mb-4 text-sm ui-text-muted cursor-pointer select-none">
                <input
                    type="checkbox"
                    checked={autoScroll}
                    onChange={event => setAutoScroll(event.target.checked)}
                />
                {t('transcript-auto-scroll')}
            </label>
        )}

        <ul className="space-y-2 text-sm md:text-base leading-7">
            {segments.map(segment => {
                const isActive = hasTimestamps && segment.idx === activeSegmentIdx
                return <li
                    key={segment.idx}
                    ref={isActive ? activeSegmentRef : undefined}
                    className={`flex gap-3 ${segment.startMs != null ? 'cursor-pointer' : ''} ${isActive ? 'ui-text-accent' : 'ui-text'}`}
                    onClick={async () => {
                        if (segment.startMs == null) {
                            return
                        }
                        await startAudioPlayer(podcastEpisode.local_url, segment.startMs / 1000)
                    }}
                >
                    {segment.startMs != null && (
                        <span className="shrink-0 tabular-nums ui-text-muted w-12 text-right">
                            {formatTimestamp(segment.startMs)}
                        </span>
                    )}
                    <span className="break-words [overflow-wrap:anywhere]">
                        {segment.speaker && <b>{segment.speaker}: </b>}
                        {segment.text}
                    </span>
                </li>
            })}
        </ul>
    </div>
}
