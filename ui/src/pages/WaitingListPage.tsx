import {useTranslation} from 'react-i18next'
import {useQueryClient} from "@tanstack/react-query"
import {$api} from "../utils/http"
import {components} from "../../schema"
import {Heading1} from "../components/Heading1"
import {Loading} from "../components/Loading"
import {EpisodeTriageRow} from "../components/EpisodeTriageRow"
import {useSnackbar} from "@/utils/toast"

type Item = components["schemas"]["PodcastEpisodeWithHistory"]
const WAITING_LIST_KEY = ['get', '/api/v1/episodes/waiting-list'] as const

export const WaitingListPage = () => {
    const {t} = useTranslation()
    const {enqueueSnackbar} = useSnackbar()
    const queryClient = useQueryClient()
    const waiting = $api.useQuery('get', '/api/v1/episodes/waiting-list')
    const triageMutation = $api.useMutation('put', '/api/v1/episodes/{id}/triage')

    const triage = async (item: Item, status: 'archived' | 'dismissed') => {
        await triageMutation.mutateAsync({
            params: {path: {id: item.podcastEpisode.id}},
            body: {status},
        })
        queryClient.setQueryData(WAITING_LIST_KEY, (old?: Item[]) =>
            (old ?? []).filter((i) => i.podcastEpisode.id !== item.podcastEpisode.id)
        )
        enqueueSnackbar(t(status === 'archived' ? 'episode-archived' : 'episode-dismissed'), {
            variant: 'success',
        })
    }

    return (
        <div>
            <Heading1>{t('waiting-list')}</Heading1>
            <p className="text-sm ui-text-muted mt-1 mb-8">{t('waiting-list-subtitle')}</p>

            {waiting.isLoading ? (
                <Loading />
            ) : !waiting.data || waiting.data.length === 0 ? (
                <p className="ui-text-muted">{t('waiting-list-empty')}</p>
            ) : (
                waiting.data.map((item) => (
                    <EpisodeTriageRow
                        key={item.podcastEpisode.id}
                        item={item}
                        onArchive={() => triage(item, 'archived')}
                        onDismiss={() => triage(item, 'dismissed')}
                    />
                ))
            )}
        </div>
    )
}
