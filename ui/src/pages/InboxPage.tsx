import {useCallback, useEffect, useState} from 'react'
import {useTranslation} from 'react-i18next'
import {$api} from "../utils/http"
import {components} from "../../schema"
import {Heading1} from "../components/Heading1"
import {EpisodeTriageRow} from "../components/EpisodeTriageRow"
import {InfiniteScrollSentinel} from "../components/InfiniteScrollSentinel"
import {ConfirmModal} from "../components/ConfirmModal"
import {CustomButtonPrimary} from "../components/CustomButtonPrimary"
import {useSnackbar} from "@/utils/toast"

type Item = components["schemas"]["PodcastEpisodeWithHistory"]
const PAGE_SIZE = 30

export const InboxPage = () => {
    const {t} = useTranslation()
    const {enqueueSnackbar} = useSnackbar()
    const [items, setItems] = useState<Item[]>([])
    const [exhausted, setExhausted] = useState(false)
    const [loaded, setLoaded] = useState(false)
    const [confirmOpen, setConfirmOpen] = useState(false)
    const fetchInbox = $api.useMutation('get', '/api/v1/episodes/inbox')
    const triageMutation = $api.useMutation('put', '/api/v1/episodes/{id}/triage')
    const clearMutation = $api.useMutation('post', '/api/v1/episodes/inbox/clear')

    const loadPage = useCallback((cursor?: string) => {
        fetchInbox
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

    const triage = async (item: Item, status: 'queued' | 'dismissed') => {
        await triageMutation.mutateAsync({
            params: {path: {id: item.podcastEpisode.id}},
            body: {status},
        })
        setItems((prev) => prev.filter((i) => i.podcastEpisode.id !== item.podcastEpisode.id))
        enqueueSnackbar(t(status === 'queued' ? 'episode-queued' : 'episode-dismissed'), {
            variant: 'success',
        })
    }

    const clearInbox = async () => {
        setConfirmOpen(false)
        await clearMutation.mutateAsync({})
        setItems([])
        setExhausted(true)
        enqueueSnackbar(t('inbox-cleared'), {variant: 'success'})
    }

    return (
        <div>
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-8">
                <div>
                    <Heading1>{t('inbox')}</Heading1>
                    <p className="text-sm ui-text-muted mt-1">{t('inbox-subtitle')}</p>
                </div>
                {items.length > 0 && (
                    <CustomButtonPrimary onClick={() => setConfirmOpen(true)}>
                        {t('clear-inbox')}
                    </CustomButtonPrimary>
                )}
            </div>

            {loaded && items.length === 0 ? (
                <p className="ui-text-muted">{t('inbox-empty')}</p>
            ) : (
                <div>
                    {items.map((item) => (
                        <EpisodeTriageRow
                            key={item.podcastEpisode.id}
                            item={item}
                            onQueue={() => triage(item, 'queued')}
                            onDismiss={() => triage(item, 'dismissed')}
                        />
                    ))}
                    <InfiniteScrollSentinel
                        className="h-1 w-full"
                        onEnter={() =>
                            loadPage(items[items.length - 1]?.podcastEpisode.date_of_recording)
                        }
                        disabled={fetchInbox.isPending || exhausted || items.length === 0}
                    />
                </div>
            )}

            <ConfirmModal
                open={confirmOpen}
                onOpenChange={setConfirmOpen}
                headerText={t('clear-inbox')}
                bodyText={t('clear-inbox-confirm')}
                acceptText={t('clear-inbox')}
                rejectText={t('cancel')}
                onAccept={clearInbox}
            />
        </div>
    )
}
