import { FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSnackbar } from 'notistack'
import { $api } from '../utils/http'
import { components } from '../../schema'
import { ADMIN_ROLE } from '../models/constants'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { Chip } from './Chip'

type CastDevice = components['schemas']['CastDeviceResponse']
type DiscoveredDevice = components['schemas']['DiscoveredCastDeviceResponse']

const formatLastSeen = (value: string | null | undefined) => {
    if (!value) return ''
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return date.toLocaleString()
}

export const SettingsCastDevices: FC = () => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const [showAgentInfo, setShowAgentInfo] = useState(false)
    const [discovered, setDiscovered] = useState<DiscoveredDevice[] | null>(null)

    const me = $api.useQuery('get', '/api/v1/users/{username}', {
        params: { path: { username: 'me' } },
    })
    const isAdmin = me.data?.role === ADMIN_ROLE

    const devicesQuery = $api.useQuery('get', '/api/v1/cast/devices')
    const discoverMutation = $api.useMutation('post', '/api/v1/cast/devices/discover')

    const devices: CastDevice[] = devicesQuery.data ?? []

    const runDiscover = async () => {
        try {
            const result = await discoverMutation.mutateAsync({})
            setDiscovered(result)
            if (result.length === 0) {
                enqueueSnackbar(t('cast-discover-empty'), { variant: 'info' })
            }
        } catch {
            // surfaced by middleware
        }
    }

    const apiKey = me.data?.apiKey ?? 'YOUR_API_KEY'
    const agentSnippet = `podfetch --agent --remote https://your.podfetch.example --api-key ${apiKey}`

    return (
        <div className="flex flex-col gap-6 ui-text">
            <div>
                <p className="text-sm ui-text-muted mb-3">
                    {t('cast-section-description')}
                </p>
            </div>

            {devicesQuery.isLoading ? (
                <div className="text-sm ui-text-muted">{t('loading')}</div>
            ) : devices.length === 0 ? (
                <div className="ui-surface rounded-md p-4 text-sm ui-text-muted">
                    {t('cast-no-devices-empty-state')}
                </div>
            ) : (
                <ul className="flex flex-col gap-2">
                    {devices.map((device) => (
                        <li
                            key={device.chromecast_uuid}
                            className="ui-surface rounded-md p-3 grid grid-cols-[auto_1fr_auto] items-center gap-3"
                        >
                            <span className="material-symbols-outlined text-2xl">cast</span>
                            <div className="flex flex-col">
                                <span className="font-medium">{device.name}</span>
                                <span className="text-xs ui-text-muted">
                                    {device.ip ? `${device.ip} · ` : ''}
                                    {device.last_seen_at ? `${t('cast-last-seen')}: ${formatLastSeen(device.last_seen_at)}` : ''}
                                </span>
                                {device.agent_id && (
                                    <span className="text-xs ui-text-muted mt-1">
                                        {t('cast-agent-id')}: <code>{device.agent_id}</code>
                                    </span>
                                )}
                            </div>
                            <Chip index={device.kind === 'chromecast_shared' ? 0 : 4}>
                                {device.kind === 'chromecast_shared' ? t('cast-kind-shared') : t('cast-kind-personal')}
                            </Chip>
                        </li>
                    ))}
                </ul>
            )}

            {isAdmin && (
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2">
                        <CustomButtonPrimary loading={discoverMutation.isPending} onClick={runDiscover}>
                            {t('cast-discover')}
                        </CustomButtonPrimary>
                        <span className="text-xs ui-text-muted">{t('cast-discover-informational')}</span>
                    </div>
                    {discovered !== null && (
                        <div className="ui-surface rounded-md p-3">
                            <h3 className="text-sm font-semibold mb-2">{t('cast-discovered-results')}</h3>
                            {discovered.length === 0 ? (
                                <p className="text-sm ui-text-muted">{t('cast-discover-empty')}</p>
                            ) : (
                                <ul className="flex flex-col gap-1 text-sm">
                                    {discovered.map((d) => (
                                        <li key={d.uuid} className="grid grid-cols-[1fr_auto] gap-2">
                                            <span>
                                                {d.friendly_name}
                                                {d.model ? <span className="ui-text-muted"> · {d.model}</span> : null}
                                            </span>
                                            <span className="ui-text-muted text-xs">
                                                {d.ip ? `${d.ip}:${d.port}` : `:${d.port}`}
                                            </span>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </div>
                    )}
                </div>
            )}

            <div className="flex flex-col gap-2">
                <button
                    type="button"
                    className="text-left text-sm underline ui-text hover:ui-text-hover w-fit"
                    onClick={() => setShowAgentInfo((prev) => !prev)}
                >
                    {showAgentInfo ? t('cast-how-to-run-agent-hide') : t('cast-how-to-run-agent')}
                </button>
                {showAgentInfo && (
                    <div className="ui-surface rounded-md p-3 text-sm flex flex-col gap-2">
                        <p>{t('cast-how-to-run-agent-description')}</p>
                        <pre className="ui-surface-2 p-2 rounded-md overflow-x-auto text-xs">
                            <code>{agentSnippet}</code>
                        </pre>
                        <p className="text-xs ui-text-muted">
                            {t('cast-how-to-run-agent-api-key')}{' '}
                            <a className="underline" href="/ui/profile">{t('profile')}</a>.
                        </p>
                    </div>
                )}
            </div>
        </div>
    )
}
