import * as Dialog from '@radix-ui/react-dialog'
import { FC, useEffect, useState } from 'react'
import { Switcher } from './Switcher'
import { PodcastSetting } from '../models/PodcastSetting'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { useTranslation } from 'react-i18next'
import { SettingsInfoIcon } from './SettingsInfoIcon'
import { CustomInput } from './CustomInput'
import { CustomSelect } from './CustomSelect'
import { options } from './SettingsNaming'
import { components } from '../../schema'
import { $api } from '../utils/http'
import { generatePodcastDefaultSettings } from '../models/PodcastDefaultSettings'
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import { ConfirmModal } from './ConfirmModal'
import { enqueueSnackbar } from 'notistack'

type PodcastSettingsModalProps = {
    podcast: components['schemas']['PodcastDto']
}

export const PodcastSettingsModal: FC<PodcastSettingsModalProps> = ({
                                                                        podcast
                                                                    }) => {
    const { t } = useTranslation()

    const settingsQuery = $api.useQuery(
        'get',
        '/api/v1/podcasts/{id}/settings',
        {
            params: { path: { id: podcast.id } },
            retry: false
        }
    )

    const updateSettings = $api.useMutation(
        'put',
        '/api/v1/podcasts/{id}/settings'
    )
    const runCleanupMutation = $api.useMutation('put', '/api/v1/settings/runcleanup')

    const downloadAllMissingMutation = $api.useMutation(
        'post',
        '/api/v1/podcasts/{id}/episodes/download-all'
    )
    const resyncFilesMutation = $api.useMutation(
        'post',
        '/api/v1/podcasts/{id}/episodes/resync-files'
    )
    const resyncDbMutation = $api.useMutation(
        'post',
        '/api/v1/podcasts/{id}/episodes/resync-db'
    )
    const deleteAllDownloadsMutation = $api.useMutation(
        'delete',
        '/api/v1/podcasts/{id}/episodes/downloads'
    )

    type ConfirmState = {
        headerText: string
        bodyText: string
        acceptText: string
        rejectText: string
        onAccept: () => void
    }
    const [confirmOpen, setConfirmOpen] = useState(false)
    const [confirmData, setConfirmData] = useState<ConfirmState | null>(null)
    const [activeTab, setActiveTab] = useState<'settings' | 'actions'>('settings')

    const [draft, setDraft] = useState<PodcastSetting | null>(null)

    useEffect(() => {
        if (settingsQuery.data) {
            setDraft(settingsQuery.data)
        } else if (!settingsQuery.isLoading) {
            setDraft(generatePodcastDefaultSettings(podcast.id))
        }
    }, [settingsQuery.data, settingsQuery.isLoading, podcast.id])

    const update = <K extends keyof PodcastSetting>(
        key: K,
        value: PodcastSetting[K]
    ) => {
        if (!draft) return
        setDraft({ ...draft, [key]: value })
    }

    const save = () => {
        if (!draft) return

        updateSettings.mutate({
            body: draft,
            params: { path: { id: podcast.id } }
        })
    }

    if (!draft) {
        return null
    }

    return (
        <Dialog.Root>
            <Dialog.Trigger asChild>
                <button className="material-symbols-outlined inline cursor-pointer align-middle ui-icon hover:ui-icon-hover">
                    settings
                </button>
            </Dialog.Trigger>

            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />

                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div
                        onClick={(e) => e.stopPropagation()}
                        className="relative ui-surface max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]"
                    >
                        <Dialog.Title className="ui-text-accent text-2xl">
                            {t('settings')}
                        </Dialog.Title>

                        <Dialog.Description className="ui-text">
                            {t('settings-configure')}
                        </Dialog.Description>

                        <Dialog.Close className="top-5 absolute right-5">
                            <span className="material-symbols-outlined">
                                close
                            </span>
                        </Dialog.Close>

                        <hr className="mb-5 mt-1 ui-border" />

                        <div className="flex gap-2 mb-5 border-b ui-border" role="tablist">
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === 'settings'}
                                onClick={() => setActiveTab('settings')}
                                className={`px-4 py-2 -mb-px border-b-2 transition-colors cursor-pointer ${
                                    activeTab === 'settings'
                                        ? 'border-(--accent-color) text-(--accent-color)'
                                        : 'border-transparent ui-text hover:text-(--accent-color-hover)'
                                }`}
                            >
                                {t('settings')}
                            </button>
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === 'actions'}
                                onClick={() => setActiveTab('actions')}
                                className={`px-4 py-2 -mb-px border-b-2 transition-colors cursor-pointer ${
                                    activeTab === 'actions'
                                        ? 'border-(--accent-color) text-(--accent-color)'
                                        : 'border-transparent ui-text hover:text-(--accent-color-hover)'
                                }`}
                            >
                                {t('batch-actions')}
                            </button>
                        </div>

                        {activeTab === 'settings' && (
                        <div className="grid grid-cols-3 gap-5">
                            <label className="col-span-2 ui-text">
                                {t('episode-numbering')}
                            </label>
                            <Switcher
                                checked={draft.episodeNumbering}
                                onChange={(v) =>
                                    update('episodeNumbering', v)
                                }
                            />

                            <label className="ui-text">
                                {t('auto-cleanup')}
                            </label>
                            <CustomButtonSecondary
                                onClick={() => runCleanupMutation.mutate({})}
                            >
                                {t('run-cleanup')}
                            </CustomButtonSecondary>
                            <Switcher
                                checked={draft.autoCleanup}
                                onChange={(v) =>
                                    update('autoCleanup', v)
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('days-to-keep')}
                                <SettingsInfoIcon
                                    headerKey="days-to-keep"
                                    textKey="days-to-keep-explanation"
                                />
                            </label>
                            <CustomInput
                                type="number"
                                value={draft.autoCleanupDays}
                                onChange={(e) =>
                                    update(
                                        'autoCleanupDays',
                                        Number(e.target.value)
                                    )
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('auto-update')}
                            </label>
                            <Switcher
                                checked={draft.autoUpdate}
                                onChange={(v) =>
                                    update('autoUpdate', v)
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('auto-download')}
                            </label>
                            <Switcher
                                checked={draft.autoDownload}
                                onChange={(v) =>
                                    update('autoDownload', v)
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('colon-replacement')}
                                <SettingsInfoIcon
                                    headerKey="colon-replacement"
                                    textKey="colon-replacement-explanation"
                                />
                            </label>
                            <CustomSelect
                                options={options}
                                value={draft.replacementStrategy}
                                onChange={(v) =>
                                    update('replacementStrategy', v)
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('use-one-cover-for-all-episodes')}
                                <SettingsInfoIcon
                                    headerKey="use-one-cover-for-all-episodes"
                                    textKey="use-one-cover-for-all-episodes-explanation"
                                />
                            </label>
                            <Switcher
                                checked={draft.useOneCoverForAllEpisodes}
                                onChange={(v) =>
                                    update('useOneCoverForAllEpisodes', v)
                                }
                            />

                            <label className="col-span-2 ui-text">
                                {t('activated')}
                            </label>
                            <Switcher
                                checked={draft.activated}
                                onChange={(v) =>
                                    update('activated', v)
                                }
                            />
                        </div>
                        )}

                        {activeTab === 'actions' && (
                        <div className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-3 items-center">
                            <div className="ui-text">
                                <div>{t('download-all-missing')}</div>
                                <div className="text-xs ui-text-muted">
                                    {t('download-all-missing-explanation')}
                                </div>
                            </div>
                            <CustomButtonSecondary
                                onClick={() => {
                                    downloadAllMissingMutation.mutate(
                                        { params: { path: { id: String(podcast.id) } } },
                                        {
                                            onSuccess: () => {
                                                enqueueSnackbar(t('download-all-missing'), { variant: 'success' })
                                            },
                                        }
                                    )
                                }}
                            >
                                {t('run')}
                            </CustomButtonSecondary>

                            <div className="ui-text">
                                <div>{t('redownload-missing-files')}</div>
                                <div className="text-xs ui-text-muted">
                                    {t('redownload-missing-files-explanation')}
                                </div>
                            </div>
                            <CustomButtonSecondary
                                onClick={() => {
                                    resyncFilesMutation.mutate(
                                        { params: { path: { id: String(podcast.id) } } },
                                        {
                                            onSuccess: () => {
                                                enqueueSnackbar(t('redownload-missing-files'), { variant: 'success' })
                                            },
                                        }
                                    )
                                }}
                            >
                                {t('run')}
                            </CustomButtonSecondary>

                            <div className="ui-text">
                                <div>{t('refresh-local-state')}</div>
                                <div className="text-xs ui-text-muted">
                                    {t('refresh-local-state-explanation')}
                                </div>
                            </div>
                            <CustomButtonSecondary
                                onClick={() => {
                                    setConfirmData({
                                        headerText: t('confirm-refresh-local-state-header'),
                                        bodyText: t('confirm-refresh-local-state-body'),
                                        acceptText: t('run'),
                                        rejectText: t('cancel'),
                                        onAccept: () => {
                                            resyncDbMutation.mutate(
                                                { params: { path: { id: String(podcast.id) } } },
                                                {
                                                    onSuccess: (data) => {
                                                        enqueueSnackbar(
                                                            t('affected-n-episodes', { count: data.affected }),
                                                            { variant: 'success' }
                                                        )
                                                    },
                                                }
                                            )
                                            setConfirmOpen(false)
                                        },
                                    })
                                    setConfirmOpen(true)
                                }}
                            >
                                {t('run')}
                            </CustomButtonSecondary>

                            <div className="ui-text">
                                <div>{t('delete-all-downloads')}</div>
                                <div className="text-xs ui-text-muted">
                                    {t('delete-all-downloads-explanation')}
                                </div>
                            </div>
                            <CustomButtonSecondary
                                className="border-(--danger-fg-color)! text-(--danger-fg-color)! hover:border-(--danger-fg-color-hover)! hover:text-(--danger-fg-color-hover)! hover:shadow-[0_2px_16px_var(--danger-fg-color-hover)]!"
                                onClick={() => {
                                    setConfirmData({
                                        headerText: t('confirm-delete-all-downloads-header'),
                                        bodyText: t('confirm-delete-all-downloads-body'),
                                        acceptText: t('delete'),
                                        rejectText: t('cancel'),
                                        onAccept: () => {
                                            deleteAllDownloadsMutation.mutate(
                                                { params: { path: { id: String(podcast.id) } } },
                                                {
                                                    onSuccess: (data) => {
                                                        enqueueSnackbar(
                                                            t('affected-n-episodes', { count: data.affected }),
                                                            { variant: 'success' }
                                                        )
                                                    },
                                                }
                                            )
                                            setConfirmOpen(false)
                                        },
                                    })
                                    setConfirmOpen(true)
                                }}
                            >
                                {t('delete')}
                            </CustomButtonSecondary>
                        </div>
                        )}

                        {confirmData && (
                            <ConfirmModal
                                open={confirmOpen}
                                onOpenChange={setConfirmOpen}
                                {...confirmData}
                            />
                        )}

                        <div className="mt-6 flex justify-end gap-3">
                            <Dialog.Close asChild>
                                <button className="ui-text">
                                    {t('cancel')}
                                </button>
                            </Dialog.Close>

                            {activeTab === 'settings' && (
                                <CustomButtonPrimary onClick={save}>
                                    {t('save')}
                                </CustomButtonPrimary>
                            )}
                        </div>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
