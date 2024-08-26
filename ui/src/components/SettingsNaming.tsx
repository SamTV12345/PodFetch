import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Controller, SubmitHandler, useForm } from 'react-hook-form'
import axios, { AxiosResponse } from 'axios'
import { enqueueSnackbar } from 'notistack'
import { Setting } from '../models/Setting'
import { UpdateNameSettings } from '../models/UpdateNameSettings'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { CustomInput } from './CustomInput'
import { Loading } from './Loading'
import { CustomCheckbox } from './CustomCheckbox'
import { SettingsInfoIcon } from './SettingsInfoIcon'
import {EpisodeFormatModal} from "./EpisodeFormatModal";
import {useDebounce} from "../utils/useDebounce";

type SettingsProps = {
    intialSettings: Setting
}

export const options = [
    {
        translationKey: 'dash-separated',
        value: 'replace-with-dash'
    },
    {
        translationKey: 'dash-separated-with-space',
        value: 'replace-with-dash-and-underscore'
    },
    {
        translationKey: 'remove',
        value: 'remove'
    }
]

export const SettingsNaming: FC = () => {
    const [settings, setSettings] = useState<Setting>()

    /* Fetch existing settings */
    useEffect(() => {
        axios.get('/settings')
            .then((res:AxiosResponse<Setting>) => {
                setSettings(res.data)
            })
    }, [])

    if (settings === undefined) {
        return <Loading />
    }

    return <Settings intialSettings={settings} />
}

const Settings: FC<SettingsProps> = ({ intialSettings }) => {
    const { t } = useTranslation()
    const [infoEpisodeModalOpen, setInfoEpisodeModalOpen] = useState<boolean>(false)
    const [infoPodcastModalOpen, setInfoPodcastModalOpen] = useState<boolean>(false)
    const [resultingPodcastFormat, setResultingPodcastFormat] = useState<string>('')
    const [resultingEpisodeFormat, setResultingEpisodeFormat] = useState<string>('')
    const { control, formState: {}, handleSubmit, watch}
        = useForm<UpdateNameSettings>({
        defaultValues: {
            replacementStrategy: intialSettings.replacementStrategy,
            episodeFormat: intialSettings.episodeFormat,
            replaceInvalidCharacters: intialSettings.replaceInvalidCharacters,
            useExistingFilename: intialSettings.useExistingFilename,
            podcastFormat: intialSettings.podcastFormat,
            directPaths: intialSettings.directPaths
        }
    })

    const episodeFormat = watch('episodeFormat')
    const podcastFormat = watch('podcastFormat')


    useDebounce(()=>{
        const content = {
            content: episodeFormat
        }
        axios.post( '/episodes/formatting', content)
            .then((v: AxiosResponse<string>)=>setResultingEpisodeFormat(v.data))
            .catch(e=>setResultingEpisodeFormat(e.response.data.error))

    },2000,[episodeFormat])

    useDebounce(()=>{
        const content = {
            content: podcastFormat
        }
        axios.post( '/podcasts/formatting', content)
            .then((v: AxiosResponse<string>)=>setResultingPodcastFormat(v.data))
            .catch(e=>setResultingPodcastFormat(e.response.data.error))
    },2000,[podcastFormat])

    const update_settings: SubmitHandler<UpdateNameSettings> = (data) => {
        axios.put(  '/settings/name', data satisfies UpdateNameSettings)
            .then(() => {
                enqueueSnackbar(t('settings-saved'), { variant: 'success' })
            })
            .catch(e=>{
                enqueueSnackbar(e.response.data.error, { variant: 'error' })
            })
    }

    return (
        <>
            <EpisodeFormatModal heading={t('standard-episode-format')} open={infoEpisodeModalOpen} setOpen={(v) => setInfoEpisodeModalOpen(v)}>
                <ul className="list-disc text-[--fg-color]">
                    <li>{'{title}'}</li>
                    <li>{'{date}'}</li>
                    <li>{'{description}'}</li>
                    <li>{'{duration}'}</li>
                    <li>{'{guid}'}</li>
                    <li>{'{url}'}</li>
                </ul>
            </EpisodeFormatModal>
            <EpisodeFormatModal heading={t('standard-podcast-format')} open={infoPodcastModalOpen} setOpen={(v) => setInfoPodcastModalOpen(v)}>
                <ul className="list-disc text-[--fg-color]">
                    <li>{'{title}'}</li>
                    <li>{'{description}'}</li>
                    <li>{'{language}'}</li>
                    <li>{'{explicit}'}</li>
                    <li>{'{keywords}'}</li>
                </ul>
            </EpisodeFormatModal>
            <form onSubmit={handleSubmit(update_settings)}>
                <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6 mb-10">
                    <fieldset className="xs:contents mb-4">
                        <legend className="self-start mb-2 xs:mb-0 text-[--fg-color]">{t('rename-podcasts')}</legend>

                        <div className="flex flex-col gap-2">
                            <div className="flex">
                                <Controller
                                    name="useExistingFilename"
                                    control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomCheckbox id="use-existing-filenames" name={name} onChange={onChange}
                                                        value={value}/>
                                    )}/>

                                <label className="ml-2 text-sm text-[--fg-secondary-color]"
                                       htmlFor="use-existing-filenames">{t('use-existing-filenames')}</label>
                            </div>
                            <div className="flex">
                                <Controller
                                    name="replaceInvalidCharacters"
                                    control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomCheckbox id="replace-invalid-characters" name={name} onChange={onChange}
                                                        value={value}/>
                                    )}/>

                                <label className="ml-2 text-sm text-[--fg-secondary-color]"
                                       htmlFor="replace-invalid-characters">{t('replace-invalid-characters-description')}</label>
                            </div>
                        </div>
                    </fieldset>

                    <div className="flex flex-col gap-2 xs:contents mb-4">
                        <label className="text-[--fg-color] flex gap-1"
                               htmlFor="colon-replacement">{t('colon-replacement')}
                            <SettingsInfoIcon headerKey="colon-replacement" textKey="auto-download-explanation"/>
                        </label>

                        <Controller
                            name="replacementStrategy"
                            control={control}
                            render={({field: {name, onChange, value}}) => (
                                <CustomSelect id="colon-replacement" name={name} options={options} onChange={onChange}
                                              value={value}/>
                            )}/>
                    </div>

                    <div className="flex flex-col gap-2 xs:contents mb-4">
                        <label className="text-[--fg-color] flex gap-1"
                               htmlFor="episode-format">{t('standard-episode-format')}
                            <button type="button">
                                    <span
                                        className="material-symbols-outlined pointer active:scale-95"
                                        onClick={() => {
                                            setInfoEpisodeModalOpen(true)
                                        }}
                                    >info</span>
                            </button>
                        </label>

                        <Controller
                            name="episodeFormat"
                            control={control}
                            render={({field: {name, onChange, value}}) => (
                                <CustomInput id="episode-format" name={name} onChange={onChange} value={value}/>
                            )}/>
                    </div>

                    <div className="flex flex-col gap-2 xs:contents mb-4">
                        <span className="text-[--fg-color]">Sample episode format</span>
                        <CustomInput value={resultingEpisodeFormat} disabled={true}></CustomInput>
                    </div>

                    <div className="flex flex-col gap-2 xs:contents mb-4">
                        <label className="text-[--fg-color] flex gap-1"
                               htmlFor="podcast-format">{t('standard-podcast-format')}
                            <button type="button">
                                    <span
                                        className="material-symbols-outlined pointer active:scale-95"
                                        onClick={() => setInfoPodcastModalOpen(true)}
                                    >info</span>
                            </button>
                        </label>

                        <Controller
                            name="podcastFormat"
                            control={control}
                            render={({field: {name, onChange, value}}) => (
                                <CustomInput id="podcast-format" name={name} onChange={onChange} value={value}/>
                            )}/>
                    </div>

                    <div className="flex flex-col gap-2 xs:contents mb-4">
                        <span className="text-[--fg-color]">Sample podcast format</span>
                        <CustomInput value={resultingPodcastFormat} disabled={true}></CustomInput>
                    </div>


                    <fieldset className="xs:contents mb-4">
                        <legend className="self-start mb-2 xs:mb-0 text-[--fg-color]">{t('use-direct-paths')}</legend>

                        <div className="flex flex-col gap-2">
                            <div className="flex">
                                <Controller
                                    name="directPaths"
                                    control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomCheckbox id="directPaths" name={name} onChange={onChange} value={value}/>
                                    )}/>
                            </div>
                        </div>
                    </fieldset>
                </div>
                <CustomButtonPrimary className="float-right" type="submit">{t('save')}</CustomButtonPrimary>
            </form>
        </>
    )
}
