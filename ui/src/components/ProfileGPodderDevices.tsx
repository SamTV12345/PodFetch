import { FC } from 'react'
import { Laptop, Monitor, Server, Smartphone, TabletSmartphone } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { components } from '../../schema'
import { Switcher } from './Switcher'

type GPodderDevice = components['schemas']['GPodderDeviceResponse']

const DeviceIcon: FC<{ kind: string }> = ({ kind }) => {
    switch (kind) {
        case 'mobile':
            return <Smartphone size={22} />
        case 'laptop':
            return <Laptop size={22} />
        case 'desktop':
            return <Monitor size={22} />
        case 'server':
            return <Server size={22} />
        default:
            return <TabletSmartphone size={22} />
    }
}

/**
 * Lets the current user choose which of their GPodder devices (AntennaPod,
 * gPodder, Kasts, …) share subscriptions with the web interface.
 */
export const ProfileGPodderDevices: FC = () => {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const devicesQuery = $api.useQuery('get', '/api/v1/gpodder/devices')
    const toggleMutation = $api.useMutation('put', '/api/v1/gpodder/devices/{device_id}/web-sync')

    const devices: GPodderDevice[] = devicesQuery.data ?? []

    const toggle = async (device: GPodderDevice, enabled: boolean) => {
        try {
            const updated = await toggleMutation.mutateAsync({
                params: { path: { device_id: device.deviceId } },
                body: { enabled },
            })
            queryClient.setQueryData(['get', '/api/v1/gpodder/devices'], updated)
        } catch {
            // surfaced by middleware
        }
    }

    return (
        <div className="flex flex-col gap-6 ui-text">
            <p className="text-sm ui-text-muted">{t('gpodder-devices-description')}</p>

            {devicesQuery.isLoading ? (
                <div className="text-sm ui-text-muted">{t('loading')}</div>
            ) : devices.length === 0 ? (
                <div className="ui-surface rounded-md p-4 text-sm ui-text-muted">
                    {t('gpodder-devices-empty-state')}
                </div>
            ) : (
                <ul className="flex flex-col gap-2">
                    {devices.map((device) => (
                        <li
                            key={device.deviceId}
                            className="ui-surface rounded-md p-3 grid grid-cols-[auto_1fr_auto] items-center gap-3"
                        >
                            <DeviceIcon kind={device.kind} />
                            <div className="flex flex-col min-w-0">
                                <span className="font-medium truncate">{device.name}</span>
                                {device.name !== device.deviceId && (
                                    <span className="text-xs ui-text-muted truncate">{device.deviceId}</span>
                                )}
                            </div>
                            <label
                                className="flex items-center gap-2 text-sm"
                                htmlFor={`gpodder-web-sync-${device.deviceId}`}
                            >
                                <span className="ui-text-muted hidden sm:inline">
                                    {t('gpodder-devices-share-with-web')}
                                </span>
                                <Switcher
                                    id={`gpodder-web-sync-${device.deviceId}`}
                                    checked={device.syncedWithWeb}
                                    disabled={toggleMutation.isPending}
                                    onChange={(enabled) => toggle(device, enabled)}
                                />
                            </label>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    )
}
