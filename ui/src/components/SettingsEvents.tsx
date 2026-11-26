import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { useSnackbar } from '../utils/toast'
import { Chip } from './Chip'
import { CustomSelect, Option } from './CustomSelect'
import { LoadingSkeletonSpan } from './ui/LoadingSkeletonSpan'

type DeliveryStatus = 'pending' | 'processing' | 'done' | 'failed' | 'dead'

const STATUSES: DeliveryStatus[] = ['dead', 'failed', 'pending', 'processing', 'done']

/** Chip colour per status: dead red, failed slate, done green. */
const STATUS_CHIP: Record<DeliveryStatus, number> = {
    dead: 3,
    failed: 1,
    pending: 2,
    processing: 0,
    done: 4,
}

const ALL = 'all'
const REFRESH_MS = 10_000

const statusOptions: Option[] = [
    { value: ALL, translationKey: 'events-status-all' },
    ...STATUSES.map((status) => ({ value: status, translationKey: `events-status-${status}` })),
]

const formatDate = (iso: string, locale: string) =>
    new Date(iso).toLocaleString(locale, { dateStyle: 'short', timeStyle: 'medium' })

export const SettingsEvents = () => {
    const { t, i18n } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const queryClient = useQueryClient()
    const [status, setStatus] = useState<string>('dead')

    const stats = $api.useQuery('get', '/api/v1/events/stats', {}, { refetchInterval: REFRESH_MS })
    const deliveries = $api.useQuery(
        'get',
        '/api/v1/events/deliveries',
        { params: { query: { status: status === ALL ? undefined : status, limit: 200 } } },
        { refetchInterval: REFRESH_MS },
    )
    const retry = $api.useMutation('post', '/api/v1/events/deliveries/{event_seq}/{handler}/retry')

    const onRetry = async (eventSeq: number, handler: string) => {
        try {
            await retry.mutateAsync({ params: { path: { event_seq: eventSeq, handler } } })
            enqueueSnackbar(t('events-retry-queued'), { variant: 'success' })
            await Promise.all([
                queryClient.invalidateQueries({ queryKey: ['get', '/api/v1/events/deliveries'] }),
                queryClient.invalidateQueries({ queryKey: ['get', '/api/v1/events/stats'] }),
            ])
        } catch {
            // error already surfaced by middleware
        }
    }

    const rows = deliveries.data ?? []

    return (
        <div className="flex flex-col gap-6">
            <p className="text-sm ui-text-muted">{t('events-description')}</p>

            <div className="flex flex-wrap gap-3" data-testid="event-stats">
                {STATUSES.map((s) => (
                    <button
                        key={s}
                        type="button"
                        onClick={() => setStatus(s)}
                        className={`flex items-center gap-2 rounded-lg border ui-border px-3 py-2 text-sm ${status === s ? 'ui-border-accent' : ''}`}
                    >
                        <Chip index={STATUS_CHIP[s]}>{t(`events-status-${s}`)}</Chip>
                        <span className="ui-text font-medium tabular-nums">
                            {stats.isLoading ? '–' : (stats.data?.[s] ?? 0)}
                        </span>
                    </button>
                ))}
            </div>

            <div className="flex items-center gap-3">
                <label htmlFor="event-status" className="text-sm ui-text">{t('events-filter')}</label>
                <CustomSelect
                    id="event-status"
                    options={statusOptions}
                    value={status}
                    onChange={setStatus}
                />
            </div>

            {deliveries.isLoading ? (
                <div className="flex flex-col gap-3">
                    {Array.from({ length: 3 }).map((_, index) => (
                        <LoadingSkeletonSpan key={index} height="20px" loading />
                    ))}
                </div>
            ) : rows.length === 0 ? (
                <div className="text-sm ui-text-muted">{t('events-none')}</div>
            ) : (
                <div className="overflow-x-auto">
                    <table className="text-left text-sm w-full" data-testid="event-deliveries">
                        <thead>
                            <tr className="border-b ui-border">
                                <th className="px-2 py-3 ui-text">{t('events-occurred')}</th>
                                <th className="px-2 py-3 ui-text">{t('events-type')}</th>
                                <th className="px-2 py-3 ui-text">{t('events-handler')}</th>
                                <th className="px-2 py-3 ui-text">{t('events-status')}</th>
                                <th className="px-2 py-3 ui-text">{t('events-attempts')}</th>
                                <th className="px-2 py-3 ui-text">{t('actions')}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows.map((row) => {
                                const rowStatus = row.status as DeliveryStatus
                                const canRetry = rowStatus === 'failed' || rowStatus === 'dead'
                                return (
                                    <tr key={`${row.eventSeq}-${row.handler}`} className="border-b ui-border align-top">
                                        <td className="px-2 py-3 ui-text whitespace-nowrap">
                                            {formatDate(row.occurredAt, i18n.language)}
                                        </td>
                                        <td className="px-2 py-3 ui-text">
                                            <details>
                                                <summary className="cursor-pointer">
                                                    {row.eventType} <span className="ui-text-muted">v{row.eventVersion}</span>
                                                </summary>
                                                <pre className="mt-2 max-w-md overflow-x-auto rounded-sm p-2 text-xs ui-surface-muted">
                                                    {JSON.stringify(row.payload, null, 2)}
                                                </pre>
                                            </details>
                                        </td>
                                        <td className="px-2 py-3 ui-text">{row.handler}</td>
                                        <td className="px-2 py-3">
                                            <Chip index={STATUS_CHIP[rowStatus] ?? 5}>{t(`events-status-${rowStatus}`)}</Chip>
                                            {row.lastError && (
                                                <div className="mt-1 max-w-xs text-xs ui-text-muted break-words">{row.lastError}</div>
                                            )}
                                            {rowStatus === 'failed' && (
                                                <div className="mt-1 text-xs ui-text-muted">
                                                    {t('events-next-attempt', { time: formatDate(row.nextAttemptAt, i18n.language) })}
                                                </div>
                                            )}
                                        </td>
                                        <td className="px-2 py-3 ui-text tabular-nums">{row.attempts}</td>
                                        <td className="px-2 py-3">
                                            {canRetry && (
                                                <button
                                                    className="ui-text-accent hover:underline disabled:opacity-50"
                                                    onClick={() => onRetry(row.eventSeq, row.handler)}
                                                    disabled={retry.isPending}
                                                >
                                                    {t('events-retry')}
                                                </button>
                                            )}
                                        </td>
                                    </tr>
                                )
                            })}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    )
}
