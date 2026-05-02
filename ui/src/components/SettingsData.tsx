import { useTranslation } from 'react-i18next'
import { useSnackbar } from 'notistack'
import { Setting } from '../models/Setting'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { CustomInput } from './CustomInput'
import { Switcher } from './Switcher'
import { SettingsInfoIcon } from './SettingsInfoIcon'
import {$api} from '../utils/http'
import {useQueryClient} from "@tanstack/react-query";
import { SponsorBlockCategorySettings } from './SponsorBlockCategorySettings'


export const Settings = () => {
    const {enqueueSnackbar} = useSnackbar()
    const settingsModel = $api.useQuery('get', '/api/v1/settings')
    const runCleanupMutation = $api.useMutation('put', '/api/v1/settings/runcleanup')
    const rescanEpisodesMutation = $api.useMutation('post', '/api/v1/settings/rescan-episodes')
    const saveSettingsMutation = $api.useMutation('put', '/api/v1/settings')
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    return (
        <div>
            <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6 mb-10 ui-text">
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <div>
                        <label className="mr-6" htmlFor="auto-cleanup">{t('auto-cleanup')}</label>
                        <CustomButtonSecondary onClick={() => {
                            runCleanupMutation.mutate({})
                        }}>{t('run-cleanup')}</CustomButtonSecondary>
                    </div>
                    <Switcher checked={settingsModel.data?.autoCleanup} loading={settingsModel.isLoading} className="xs:justify-self-end" id="auto-cleanup" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData?: Setting) => ({
                            ...oldData,
                            autoCleanup: !oldData?.autoCleanup
                        }))
                    }} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="days-to-keep" className="flex gap-1">{t('days-to-keep')} <SettingsInfoIcon headerKey="days-to-keep" textKey="days-to-keep-explanation" /></label>
                    <CustomInput loading={settingsModel.isLoading} className="w-20" id="days-to-keep" onChange={(e) => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            autoCleanupDays: parseInt(e.target.value)
                        }))
                    }} type="number" value={settingsModel.data?.autoCleanupDays} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="auto-update" className="flex gap-1">{t('auto-update')} <SettingsInfoIcon headerKey="auto-update" textKey="auto-update-explanation" /></label>
                    <Switcher loading={settingsModel.isLoading} checked={settingsModel.data?.autoUpdate} className="xs:justify-self-end" id="auto-update" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            autoUpdate: !oldData?.autoUpdate
                        }))
                    }} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="auto-download" className="flex gap-1">{t('auto-download')} <SettingsInfoIcon headerKey="auto-download" textKey="auto-download-explanation" /></label>
                    <Switcher loading={settingsModel.isLoading} checked={settingsModel.data?.autoDownload} className="xs:justify-self-end" id="auto-download" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            autoDownload: !oldData?.autoDownload
                        }))
                    }} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="number-of-podcasts-to-download" className="flex gap-1">{t('number-of-podcasts-to-download')} <SettingsInfoIcon headerKey="number-of-podcasts-to-download" textKey="number-of-podcasts-to-download-explanation" /></label>
                    <CustomInput loading={settingsModel.isLoading} className="w-20" id="number-of-podcasts-to-download" onChange={(e) => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            podcastPrefill: parseInt(e.target.value)
                        }))
                    }} type="number" value={settingsModel.data?.podcastPrefill} />
                </div>
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="auto-transcode-opus" className="flex gap-1">{t('auto-transcode-opus')} <SettingsInfoIcon headerKey="auto-transcode-opus" textKey="auto-transcode-opus-explanation" /></label>
                    <Switcher loading={settingsModel.isLoading} checked={settingsModel.data?.autoTranscodeOpus} className="xs:justify-self-end" id="auto-transcode-opus" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            autoTranscodeOpus: !oldData?.autoTranscodeOpus
                        }))
                    }} />
                </div>
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="use-one-cover-for-all-episodes" className="flex gap-1">{t('use-one-cover-for-all-episodes')} <SettingsInfoIcon headerKey="use-one-cover-for-all-episodes" textKey="use-one-cover-for-all-episodes-explanation" /></label>
                    <Switcher loading={settingsModel.isLoading} checked={settingsModel.data?.useOneCoverForAllEpisodes} className="xs:justify-self-end" id="use-one-cover-for-all-episodes" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            useOneCoverForAllEpisodes: !oldData?.useOneCoverForAllEpisodes
                        }))
                    }} />
                </div>
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="sponsorblock-enabled" className="flex gap-1">{t('sponsorblock-enabled')} <SettingsInfoIcon headerKey="sponsorblock-enabled" textKey="sponsorblock-enabled-explanation" /></label>
                    <Switcher loading={settingsModel.isLoading} checked={settingsModel.data?.sponsorblockEnabled} className="xs:justify-self-end" id="sponsorblock-enabled" onChange={() => {
                        queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                            ...oldData,
                            sponsorblockEnabled: !oldData?.sponsorblockEnabled
                        }))
                    }} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label className="flex gap-1">{t('rescan-audio-files')} <SettingsInfoIcon headerKey="rescan-audio-files" textKey="rescan-audio-files-description" /></label>
                    <CustomButtonPrimary onClick={async ()=>{
                        await rescanEpisodesMutation.mutateAsync({})
                        enqueueSnackbar(t('rescan-done'), { variant: 'success' })
                    }}>{t('rescan-audio-files')}</CustomButtonPrimary>
                </div>
            </div>

            {settingsModel.data?.sponsorblockEnabled && (
                <div className="mb-10">
                    <h3 className="ui-text-accent text-lg mb-3">{t('sponsorblock-categories')}</h3>
                    <SponsorBlockCategorySettings
                        loading={settingsModel.isLoading}
                        categories={settingsModel.data?.sponsorblockCategories ?? []}
                        onChange={(next) => {
                            queryClient.setQueryData(['get', '/api/v1/settings'], (oldData: Setting) => ({
                                ...oldData,
                                sponsorblockCategories: next
                            }))
                        }}
                    />
                </div>
            )}

            <CustomButtonPrimary loading={settingsModel.isLoading} className="float-right" onClick={() => {
                saveSettingsMutation.mutateAsync({
                    body: settingsModel.data!
                }).then(() => {
                    enqueueSnackbar(t('settings-saved'), { variant: 'success' })
                })
            }}>{t('save')}</CustomButtonPrimary>
        </div>
    )
}
