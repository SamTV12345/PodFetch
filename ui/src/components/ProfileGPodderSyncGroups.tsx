import { FC, useState } from 'react'
import { Globe, Laptop, Monitor, Server, Smartphone, TabletSmartphone, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { components } from '../../schema'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { CustomCheckbox } from './CustomCheckbox'
import { CustomSelect } from './CustomSelect'

type GPodderDevice = components['schemas']['GPodderDeviceResponse']
type SyncGroups = components['schemas']['GPodderSyncGroupsResponse']

const WEB_DEVICE_KIND = 'webview'
const QUERY_KEY = ['get', '/api/v1/gpodder/sync-groups']

const DeviceIcon: FC<{ kind: string }> = ({ kind }) => {
    switch (kind) {
        case WEB_DEVICE_KIND:
            return <Globe size={20} />
        case 'mobile':
            return <Smartphone size={20} />
        case 'laptop':
            return <Laptop size={20} />
        case 'desktop':
            return <Monitor size={20} />
        case 'server':
            return <Server size={20} />
        default:
            return <TabletSmartphone size={20} />
    }
}

const DeviceLabel: FC<{ device: GPodderDevice }> = ({ device }) => {
    const { t } = useTranslation()
    const isWeb = device.kind === WEB_DEVICE_KIND
    return (
        <div className="flex items-center gap-3 min-w-0">
            <DeviceIcon kind={device.kind} />
            <div className="flex flex-col min-w-0">
                <span className="font-medium truncate">{isWeb ? t('gpodder-web-device') : device.name}</span>
                <span className="text-xs ui-text-muted truncate">
                    {isWeb ? t('gpodder-web-device-description') : device.deviceId}
                </span>
            </div>
        </div>
    )
}

/**
 * GPodder-style sync group management for the current user: devices in the
 * same group share their subscriptions; the web interface takes part as a
 * device of its own (its subscriptions are the user's favorites).
 */
export const ProfileGPodderSyncGroups: FC = () => {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [selected, setSelected] = useState<string[]>([])

    const groupsQuery = $api.useQuery('get', '/api/v1/gpodder/sync-groups')
    const updateMutation = $api.useMutation('put', '/api/v1/gpodder/sync-groups')

    const data: SyncGroups = groupsQuery.data ?? { groups: [], notSynchronized: [] }
    const groups = data.groups.map((g) => g.devices.map((d) => d.deviceId))
    const allDevices = [...data.groups.flatMap((g) => g.devices), ...data.notSynchronized]
    const deviceName = (id: string) => {
        const device = allDevices.find((d) => d.deviceId === id)
        if (!device) return id
        return device.kind === WEB_DEVICE_KIND ? t('gpodder-web-device') : device.name
    }
    const hasClientDevices = allDevices.some((d) => d.kind !== WEB_DEVICE_KIND)

    const save = async (next: string[][]) => {
        try {
            const updated = await updateMutation.mutateAsync({ body: { groups: next } })
            queryClient.setQueryData(QUERY_KEY, updated)
            setSelected((current) => current.filter((id) => updated.notSynchronized.some((d) => d.deviceId === id)))
        } catch {
            // surfaced by middleware
        }
    }

    const withoutDevice = (all: string[][], deviceId: string) =>
        all.map((group) => group.filter((id) => id !== deviceId))

    const removeDevice = (deviceId: string) => save(withoutDevice(groups, deviceId))

    const addDevice = (groupIndex: number, deviceId: string) => {
        const next = withoutDevice(groups, deviceId)
        next[groupIndex] = [...(next[groupIndex] ?? []), deviceId]
        return save(next)
    }

    const mergeGroups = (groupIndex: number, otherIndex: number) => {
        const next = groups.map((group) => [...group])
        next[groupIndex] = [...(next[groupIndex] ?? []), ...(next[otherIndex] ?? [])]
        next.splice(otherIndex, 1)
        return save(next)
    }

    const dissolveGroup = (groupIndex: number) => save(groups.filter((_, index) => index !== groupIndex))

    const createGroup = () => save([...groups, selected])

    if (groupsQuery.isLoading) {
        return <div className="text-sm ui-text-muted">{t('loading')}</div>
    }

    return (
        <div className="flex flex-col gap-6 ui-text">
            <p className="text-sm ui-text-muted">{t('gpodder-sync-groups-description')}</p>

            {data.groups.map((group, groupIndex) => {
                const addOptions = allDevices
                    .filter((d) => !group.devices.some((member) => member.deviceId === d.deviceId))
                    .map((d) => {
                        const otherGroup = groups.findIndex((g) => g.includes(d.deviceId))
                        return {
                            value: d.deviceId,
                            label: otherGroup >= 0
                                ? `${deviceName(d.deviceId)} (${t('gpodder-in-group', { number: otherGroup + 1 })})`
                                : deviceName(d.deviceId),
                        }
                    })
                const mergeOptions = data.groups
                    .map((_, index) => index)
                    .filter((index) => index !== groupIndex)
                    .map((index) => ({
                        value: String(index),
                        label: t('gpodder-sync-group', { number: index + 1 }),
                    }))

                return (
                    <section key={group.groupId} className="ui-surface rounded-md p-4 flex flex-col gap-3">
                        <div className="flex items-center justify-between gap-3">
                            <h3 className="font-semibold">{t('gpodder-sync-group', { number: groupIndex + 1 })}</h3>
                            <button
                                type="button"
                                className="text-sm underline ui-text-muted hover:ui-text disabled:opacity-50"
                                disabled={updateMutation.isPending}
                                onClick={() => dissolveGroup(groupIndex)}
                            >
                                {t('gpodder-sync-group-dissolve')}
                            </button>
                        </div>

                        <ul className="flex flex-col gap-2">
                            {group.devices.map((device) => (
                                <li key={device.deviceId} className="flex items-center justify-between gap-3">
                                    <DeviceLabel device={device} />
                                    <button
                                        type="button"
                                        title={t('gpodder-sync-group-remove-device')}
                                        aria-label={t('gpodder-sync-group-remove-device')}
                                        className="ui-text-muted hover:ui-text disabled:opacity-50 shrink-0"
                                        disabled={updateMutation.isPending}
                                        onClick={() => removeDevice(device.deviceId)}
                                    >
                                        <X size={18} />
                                    </button>
                                </li>
                            ))}
                        </ul>

                        <div className="flex flex-wrap gap-3">
                            {addOptions.length > 0 && (
                                <CustomSelect
                                    className="min-w-56"
                                    value=""
                                    placeholder={t('gpodder-sync-group-add-device')}
                                    options={addOptions}
                                    disabled={updateMutation.isPending}
                                    onChange={(deviceId) => deviceId && addDevice(groupIndex, deviceId)}
                                />
                            )}
                            {mergeOptions.length > 0 && (
                                <CustomSelect
                                    className="min-w-56"
                                    value=""
                                    placeholder={t('gpodder-sync-group-merge')}
                                    options={mergeOptions}
                                    disabled={updateMutation.isPending}
                                    onChange={(index) => index && mergeGroups(groupIndex, Number(index))}
                                />
                            )}
                        </div>
                    </section>
                )
            })}

            <section className="flex flex-col gap-3">
                <h3 className="font-semibold">{t('gpodder-not-synchronized')}</h3>
                {data.notSynchronized.length === 0 ? (
                    <p className="text-sm ui-text-muted">{t('gpodder-all-synchronized')}</p>
                ) : (
                    <ul className="flex flex-col gap-2">
                        {data.notSynchronized.map((device) => (
                            <li key={device.deviceId} className="ui-surface rounded-md p-3 flex items-center gap-3">
                                <CustomCheckbox
                                    id={`gpodder-select-${device.deviceId}`}
                                    value={selected.includes(device.deviceId)}
                                    onChange={(checked) =>
                                        setSelected((current) =>
                                            checked
                                                ? [...current, device.deviceId]
                                                : current.filter((id) => id !== device.deviceId),
                                        )
                                    }
                                />
                                <label htmlFor={`gpodder-select-${device.deviceId}`} className="min-w-0 cursor-pointer">
                                    <DeviceLabel device={device} />
                                </label>
                            </li>
                        ))}
                    </ul>
                )}
                {!hasClientDevices && (
                    <div className="ui-surface rounded-md p-4 text-sm ui-text-muted">
                        {t('gpodder-devices-empty-state')}
                    </div>
                )}
                {data.notSynchronized.length >= 2 && (
                    <div className="flex flex-wrap items-center gap-3">
                        <CustomButtonPrimary
                            disabled={selected.length < 2 || updateMutation.isPending}
                            onClick={createGroup}
                        >
                            {t('gpodder-create-group')}
                        </CustomButtonPrimary>
                        {selected.length < 2 && (
                            <span className="text-xs ui-text-muted">{t('gpodder-create-group-hint')}</span>
                        )}
                        {selected.length > 0 && (
                            <CustomButtonSecondary onClick={() => setSelected([])}>
                                {t('cancel')}
                            </CustomButtonSecondary>
                        )}
                    </div>
                )}
            </section>
        </div>
    )
}
