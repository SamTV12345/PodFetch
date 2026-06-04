import {FC, useMemo} from 'react'
import {useTranslation} from 'react-i18next'
import {Archive, CirclePlay, CloudDownload, X} from 'lucide-react'
import {components} from "../../schema"
import {removeHTML, formatTime} from '../utils/Utilities'
import useCommon from "../store/CommonSlice"
import useAudioPlayer from "../store/AudioPlayerSlice"
import {startAudioPlayer} from "../utils/audioPlayer"

type EpisodeTriageRowProps = {
    item: components["schemas"]["PodcastEpisodeWithHistory"],
    /** Queue the episode for download (inbox view). */
    onQueue?: () => void,
    /** Dismiss the episode / remove it from the list. */
    onDismiss?: () => void,
    /** Move the episode to the archive (waiting list view). */
    onArchive?: () => void,
}

export const EpisodeTriageRow: FC<EpisodeTriageRowProps> = ({item, onQueue, onDismiss, onArchive}) => {
    const {t} = useTranslation()
    const episode = item.podcastEpisode
    const history = item.podcastHistoryItem
    const setCurrentEpisodeIndex = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setCurrentEpisodes = useCommon(state => state.setSelectedEpisodes)

    const playPercentage = useMemo(() => {
        return history?.total ? (history.position ?? 0) * 100 / history.total : 0
    }, [history])

    const play = async () => {
        setCurrentEpisodes([{podcastEpisode: episode, podcastHistoryItem: history}])
        setCurrentEpisodeIndex(0)
        const startPosition = playPercentage < 98 ? (history?.position ?? 0) : 0
        await startAudioPlayer(episode.local_url, startPosition)
    }

    return (
        <div className="grid grid-cols-[auto_1fr_auto] gap-x-4 gap-y-1 items-center group mb-8">
            {/* Thumbnail */}
            <img
                src={episode.local_image_url}
                alt={episode.name}
                className="hidden xs:block self-start rounded-lg w-20 h-20 object-cover row-span-3"
            />

            {/* Meta row */}
            <div className="col-start-1 xs:col-start-2 col-end-4 flex flex-wrap items-center gap-x-4 gap-y-1">
                <span className="text-xs ui-text-muted">{formatTime(episode.date_of_recording)}</span>
                {!episode.status && (
                    <span className="text-xs ui-text-muted">{t('not-downloaded')}</span>
                )}
            </div>

            {/* Title */}
            <span className="col-start-1 xs:col-start-2 font-bold leading-tight ui-text">
                {episode.name}
            </span>

            {/* Actions */}
            <span className="row-start-1 row-span-3 col-start-3 flex items-center gap-4 self-center">
                <CirclePlay
                    size={40}
                    strokeWidth={1.5}
                    aria-label={t('play') as string}
                    className="cursor-pointer ui-text hover:ui-text-hover active:scale-90"
                    onClick={(e) => {
                        e.stopPropagation()
                        void play()
                    }}
                />
                {onQueue && (
                    episode.status
                        ? <CloudDownload size={22} aria-label={t('downloaded') as string}
                                         className="ui-icon" fill="currentColor"/>
                        : <CloudDownload
                            size={22}
                            aria-label={t('triage-queue') as string}
                            className="cursor-pointer ui-icon hover:ui-icon-hover active:scale-90"
                            onClick={(e) => {
                                e.stopPropagation()
                                onQueue()
                            }}
                        />
                )}
                {onArchive && (
                    <Archive
                        size={22}
                        aria-label={t('triage-archive') as string}
                        className="cursor-pointer ui-icon hover:ui-icon-hover active:scale-90"
                        onClick={(e) => {
                            e.stopPropagation()
                            onArchive()
                        }}
                    />
                )}
                {onDismiss && (
                    <X
                        size={22}
                        aria-label={t('triage-dismiss') as string}
                        className="cursor-pointer ui-icon hover:ui-icon-hover active:scale-90"
                        onClick={(e) => {
                            e.stopPropagation()
                            onDismiss()
                        }}
                    />
                )}
            </span>

            {/* Description */}
            <div
                className="col-start-1 xs:col-start-2 col-end-4 line-clamp-2 text-sm ui-text-muted"
                dangerouslySetInnerHTML={removeHTML(episode.description)}
            />

            {/* Progress bar */}
            {history?.total && history.position ? (
                <div className="col-start-1 xs:col-start-2 col-end-4 ui-bg-foreground h-1 rounded-full overflow-hidden">
                    <div className="ui-bg-accent h-1" style={{width: playPercentage + "%"}}/>
                </div>
            ) : null}
        </div>
    )
}
