import {useCallback, useEffect, useState} from 'react'
import {useTranslation} from 'react-i18next'
import {$api} from "../utils/http"
import {components} from "../../schema"
import {Heading1} from "../components/Heading1"
import {EpisodeTriageRow} from "../components/EpisodeTriageRow"
import {InfiniteScrollSentinel} from "../components/InfiniteScrollSentinel"

type Item = components["schemas"]["PodcastEpisodeWithHistory"]
const PAGE_SIZE = 30

export const EpisodesArchivePage = () => {
    const {t} = useTranslation()
    const [items, setItems] = useState<Item[]>([])
    const [exhausted, setExhausted] = useState(false)
    const [loaded, setLoaded] = useState(false)
    const fetchArchive = $api.useMutation('get', '/api/v1/episodes/archive')

    const loadPage = useCallback((cursor?: string) => {
        fetchArchive
            .mutateAsync({params: {query: {lastEpisodeDate: cursor, limit: PAGE_SIZE}}})
            .then((resp) => {
                setLoaded(true)
                if (!resp || resp.length === 0) {
                    if (!cursor) {
                        setItems([])
                    }
                    setExhausted(true)
                    return
                }
                setItems((prev) => (cursor ? [...prev, ...resp] : resp))
                if (resp.length < PAGE_SIZE) {
                    setExhausted(true)
                }
            })
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    useEffect(() => {
        loadPage(undefined)
    }, [loadPage])

    return (
        <div>
            <Heading1>{t('episodes-archive')}</Heading1>
            <p className="text-sm ui-text-muted mt-1 mb-8">{t('episodes-archive-subtitle')}</p>

            {loaded && items.length === 0 ? (
                <p className="ui-text-muted">{t('episodes-archive-empty')}</p>
            ) : (
                <div>
                    {items.map((item) => (
                        <EpisodeTriageRow key={item.podcastEpisode.id} item={item} />
                    ))}
                    <InfiniteScrollSentinel
                        className="h-1 w-full"
                        onEnter={() =>
                            loadPage(items[items.length - 1]?.podcastEpisode.date_of_recording)
                        }
                        disabled={fetchArchive.isPending || exhausted || items.length === 0}
                    />
                </div>
            )}
        </div>
    )
}
