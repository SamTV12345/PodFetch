import {FC, useMemo} from 'react'
import {useTranslation} from 'react-i18next'
import {Archive, CirclePlay, ListPlus, X} from 'lucide-react'
import {components} from "../../schema"
import {removeHTML, formatTime} from '../utils/Utilities'
import useCommon from "../store/CommonSlice"
import useAudioPlayer from "../store/AudioPlayerSlice"
import {startAudioPlayer} from "../utils/audioPlayer"

type EpisodeTriageRowProps = {
    item: components["schemas"]["PodcastEpisodeWithHistory"],
    /** Add the episode to the waiting list (inbox view). Downloads it first if needed. */
    onQueue?: () => void,
    /** Dismiss the episode / remove it from the list. */
    onDismiss?: () => void,
    /** Move the episode to the archive (waiting list view). */
    onArchive?: () => void,
}

const ICON_SIZE = 20

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
        <div className="flex items-start gap-4 group mb-6 border-b ui-border pb-6 last:border-b-0">
            {/* Thumbnail */}
            <img
                src={episode.local_image_url}
                alt={episode.name}
                className="hidden xs:block shrink-0 rounded-lg w-16 h-16 object-cover"
            />

            {/* Main content — `min-w-0` lets long titles/descriptions wrap and
                truncate instead of forcing the row wider than the viewport. */}
            <div className="flex-1 min-w-0">
                <span className="block text-xs ui-text-muted mb-1">
                    {formatTime(episode.date_of_recording)}
                </span>
                <span className="block font-bold leading-tight ui-text break-words group-hover:ui-text-hover">
                    {episode.name}
                </span>
                <div
                    className="mt-1 line-clamp-2 overflow-hidden text-sm ui-text-muted break-words"
                    dangerouslySetInnerHTML={removeHTML(episode.description)}
                />
                {history?.total && history.position ? (
                    <div className="mt-2 max-w-md ui-bg-foreground h-1 rounded-full overflow-hidden">
                        <div className="ui-bg-accent h-1 rounded-full" style={{width: playPercentage + "%"}}/>
                    </div>
                ) : null}
            </div>

            {/* Actions */}
            <div className="flex items-center gap-3 shrink-0 self-center">
                <CirclePlay
                    size={32}
                    strokeWidth={1.5}
                    aria-label={t('play') as string}
                    className="cursor-pointer ui-text hover:ui-text-hover active:scale-90"
                    onClick={(e) => {
                        e.stopPropagation()
                        void play()
                    }}
                />
                {onQueue && (
                    <ListPlus
                        size={ICON_SIZE}
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
                        size={ICON_SIZE}
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
                        size={ICON_SIZE}
                        aria-label={t('triage-dismiss') as string}
                        className="cursor-pointer ui-icon hover:ui-icon-hover active:scale-90"
                        onClick={(e) => {
                            e.stopPropagation()
                            onDismiss()
                        }}
                    />
                )}
            </div>
        </div>
    )
}
