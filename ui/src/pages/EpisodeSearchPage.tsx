import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Search } from 'lucide-react'
import { Heading1 } from '../components/Heading1'
import { EpisodeSearch } from '../components/EpisodeSearch'
import { CustomInput } from '../components/CustomInput'
import { Spinner } from '../components/Spinner'
import { EmptyResultIcon } from '../icons/EmptyResultIcon'
import { formatTimestamp } from '../components/PodcastEpisodeTranscript'
import { formatTime } from '../utils/Utilities'
import { useDebounce } from '../utils/useDebounce'
import { startAudioPlayer } from '../utils/audioPlayer'
import { $api } from '../utils/http'
import { components } from '../../schema'
import useCommon from '../store/CommonSlice'
import useAudioPlayer from '../store/AudioPlayerSlice'

type SearchMode = 'metadata' | 'transcripts'

/**
 * The backend snippet marks matches with plain `<b>`/`</b>`. Escape everything
 * else so transcript content can never inject markup, then restore only the
 * bold tags.
 */
const snippetToSafeHtml = (snippet: string) => {
    const escaped = snippet
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
    return {
        __html: escaped
            .replaceAll('&lt;b&gt;', '<b>')
            .replaceAll('&lt;/b&gt;', '</b>')
    }
}

const TranscriptSearchResults = ({ query }: { query: string }) => {
    const { t } = useTranslation()
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const setCurrentEpisodeIndex = useAudioPlayer(state => state.setCurrentPodcastEpisode)

    const search = $api.useQuery('get', '/api/v1/transcripts/search', {
        params: {
            query: {
                q: query,
                page: 0
            }
        }
    }, {
        enabled: query.length > 0
    })

    const playEpisodeAt = async (episode: components["schemas"]["PodcastEpisodeDto"], startMs?: number | null) => {
        setSelectedEpisodes([{
            podcastEpisode: episode,
            podcastHistoryItem: null
        }])
        setCurrentEpisodeIndex(0)
        await startAudioPlayer(episode.local_url, (startMs ?? 0) / 1000)
    }

    if (query.length === 0) {
        return <div className="grid place-items-center py-4"><EmptyResultIcon /></div>
    }

    if (search.isLoading) {
        return <div className="grid place-items-center p-6"><Spinner className="w-12 h-12"/></div>
    }

    if (!search.data || search.data.length === 0) {
        return <div className="grid place-items-center py-4">
            <span className="p-3 ui-text-muted">{t('no-results-found-for')} "<span className="ui-text">{query}</span>"</span>
        </div>
    }

    return <ul className="flex min-w-0 flex-col gap-10 overflow-y-auto overflow-x-hidden my-4 px-6 py-6 scrollbox-y">
        {search.data.map(group => (
            <li className="flex min-w-0 min-h-24 gap-4 group overflow-hidden" key={group.episodeId}>
                {/* Thumbnail */}
                <img
                    alt={group.episode.name}
                    className="hidden xs:block shrink-0 rounded-lg w-32 h-32 object-cover cursor-pointer transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)]"
                    src={group.episode.image_url}
                    onClick={() => playEpisodeAt(group.episode, group.hits[0]?.startMs)}
                />

                {/* Information */}
                <div className="flex min-w-0 flex-col gap-2">
                    <span className="text-sm ui-text-muted">{formatTime(group.episode.date_of_recording)}</span>
                    <span
                        className="font-bold leading-tight ui-text cursor-pointer transition-color hover:ui-text-hover break-words [overflow-wrap:anywhere]"
                        onClick={() => playEpisodeAt(group.episode, group.hits[0]?.startMs)}
                    >
                        {group.episode.name}
                    </span>

                    {group.hits.slice(0, 3).map(hit => (
                        <div
                            className="flex items-baseline gap-2 cursor-pointer"
                            key={hit.transcriptId + (hit.startMs ?? '') + hit.snippet}
                            onClick={() => playEpisodeAt(group.episode, hit.startMs)}
                        >
                            {hit.startMs != null && (
                                <span className="shrink-0 rounded px-1.5 py-0.5 text-xs tabular-nums ui-bg-foreground ui-text-muted">
                                    {formatTimestamp(hit.startMs)}
                                </span>
                            )}
                            <span
                                className="text-sm ui-text break-words [overflow-wrap:anywhere] [&_b]:ui-text-accent"
                                dangerouslySetInnerHTML={snippetToSafeHtml(hit.snippet)}
                            />
                        </div>
                    ))}
                </div>
            </li>
        ))}
    </ul>
}

export const EpisodeSearchPage = () => {
    const { t } = useTranslation()
    const [mode, setMode] = useState<SearchMode>('metadata')
    const [transcriptSearchText, setTranscriptSearchText] = useState('')
    const [debouncedTranscriptQuery, setDebouncedTranscriptQuery] = useState('')

    useDebounce(() => setDebouncedTranscriptQuery(transcriptSearchText.trim()), 500, [transcriptSearchText])

    const modeButtonClass = (buttonMode: SearchMode) =>
        `px-4 py-2 text-sm cursor-pointer transition-colors ${mode === buttonMode ? 'ui-bg-accent text-white' : 'ui-text hover:ui-text-hover'}`

    const segmentedControl = (
        <div className="inline-flex rounded-lg border ui-border mb-6 overflow-hidden">
            <button className={modeButtonClass('metadata')} onClick={() => setMode('metadata')} type="button">
                {t('search-in-metadata')}
            </button>
            <button className={modeButtonClass('transcripts')} onClick={() => setMode('transcripts')} type="button">
                {t('search-in-transcripts')}
            </button>
        </div>
    )

    return (
        <>
            <Heading1 className="mb-10">{t('search-episodes')}</Heading1>

            {segmentedControl}

            {mode === 'metadata' ? (
                <EpisodeSearch showBlankState={true} />
            ) : (
                <>
                    <div className="flex items-center relative">
                        <CustomInput className="pl-10 w-full" id="transcript-search-input" onChange={(v) =>
                            setTranscriptSearchText(v.target.value)} placeholder={t('search-in-transcripts')!} type="text" value={transcriptSearchText} />

                        <Search size={16} className="absolute left-2 ui-input-icon" />
                    </div>

                    <TranscriptSearchResults query={debouncedTranscriptQuery} />
                </>
            )}
        </>
    )
}
