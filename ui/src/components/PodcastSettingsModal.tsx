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
import { $api, client } from '../utils/http'
import { generatePodcastDefaultSettings } from '../models/PodcastDefaultSettings'
import {CustomButtonPrimary} from "./CustomButtonPrimary";

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

    const [draft, setDraft] = useState<PodcastSetting | null>(null)

    useEffect(() => {
        if (settingsQuery.data) {
            setDraft(settingsQuery.data)
        }
    }, [settingsQuery.data])

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
                <button className="material-symbols-outlined inline cursor-pointer align-middle text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)">
                    settings
                </button>
            </Dialog.Trigger>

            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />

                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div
                        onClick={(e) => e.stopPropagation()}
                        className="relative bg-(--bg-color) max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]"
                    >
                        <Dialog.Title className="text-(--accent-color) text-2xl">
                            {t('settings')}
                        </Dialog.Title>

                        <Dialog.Description className="text-(--fg-color)">
                            {t('settings-configure')}
                        </Dialog.Description>

                        <Dialog.Close className="top-5 absolute right-5">
                            <span className="material-symbols-outlined">
                                close
                            </span>
                        </Dialog.Close>

                        <hr className="mb-5 mt-1 border-(--border-color)" />

                        <div className="grid grid-cols-3 gap-5">
                            <label className="col-span-2 text-(--fg-color)">
                                {t('episode-numbering')}
                            </label>
                            <Switcher
                                checked={draft.episodeNumbering}
                                onChange={(v) =>
                                    update('episodeNumbering', v)
                                }
                            />

                            <label className="text-(--fg-color)">
                                {t('auto-cleanup')}
                            </label>
                            <CustomButtonSecondary
                                onClick={() =>
                                    client.PUT('/api/v1/settings/runcleanup')
                                }
                            >
                                {t('run-cleanup')}
                            </CustomButtonSecondary>
                            <Switcher
                                checked={draft.autoCleanup}
                                onChange={(v) =>
                                    update('autoCleanup', v)
                                }
                            />

                            <label className="col-span-2 text-(--fg-color)">
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

                            <label className="col-span-2 text-(--fg-color)">
                                {t('auto-update')}
                            </label>
                            <Switcher
                                checked={draft.autoUpdate}
                                onChange={(v) =>
                                    update('autoUpdate', v)
                                }
                            />

                            <label className="col-span-2 text-(--fg-color)">
                                {t('auto-download')}
                            </label>
                            <Switcher
                                checked={draft.autoDownload}
                                onChange={(v) =>
                                    update('autoDownload', v)
                                }
                            />

                            <label className="col-span-2 text-(--fg-color)">
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

                            <label className="col-span-2 text-(--fg-color)">
                                {t('activated')}
                            </label>
                            <Switcher
                                checked={draft.activated}
                                onChange={(v) =>
                                    update('activated', v)
                                }
                            />
                        </div>

                        <div className="mt-6 flex justify-end gap-3">
                            <Dialog.Close asChild>
                                <button className="text-(--fg-color)">
                                    {t('cancel')}
                                </button>
                            </Dialog.Close>

                            <CustomButtonPrimary onClick={save}>
                                {t('save')}
                            </CustomButtonPrimary>
                        </div>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}