import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Controller, SubmitHandler, useForm } from 'react-hook-form'
import axios, { AxiosResponse } from 'axios'
import { enqueueSnackbar } from 'notistack'
import { apiURL } from '../utils/Utilities'
import { Setting } from '../models/Setting'
import { UpdateNameSettings } from '../models/UpdateNameSettings'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { CustomInput } from './CustomInput'
import { Loading } from './Loading'
import { CustomCheckbox } from './CustomCheckbox'
import { SettingsInfoIcon } from './SettingsInfoIcon'

type SettingsProps = {
    intialSettings: Setting
}

const options = [
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
        axios.get(apiURL + '/settings')
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

    const { control, formState: {}, handleSubmit}
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

    const update_settings: SubmitHandler<UpdateNameSettings> = (data) => {
        axios.put(apiURL + '/settings/name', data satisfies UpdateNameSettings)
            .then(() => {
                enqueueSnackbar(t('settings-saved'), { variant: 'success' })
            })
    }

    return (
        <form onSubmit={handleSubmit(update_settings)}>
            <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6 mb-10">
                <fieldset className="xs:contents mb-4">
                    <legend className="self-start mb-2 xs:mb-0 text-[--fg-color]">{t('rename-podcasts')}</legend>

                    <div className="flex flex-col gap-2">
                        <div className="flex">
                            <Controller
                            name="useExistingFilename"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomCheckbox id="use-existing-filenames" name={name} onChange={onChange} value ={value} />
                            )} />

                            <label className="ml-2 text-sm text-[--fg-secondary-color]" htmlFor="use-existing-filenames">{t('use-existing-filenames')}</label>
                        </div>
                        <div className="flex">
                            <Controller
                            name="replaceInvalidCharacters"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomCheckbox id="replace-invalid-characters" name={name} onChange={onChange} value ={value} />
                            )} />

                            <label className="ml-2 text-sm text-[--fg-secondary-color]" htmlFor="replace-invalid-characters">{t('replace-invalid-characters-description')}</label>
                        </div>
                    </div>
                </fieldset>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label className="text-[--fg-color] flex gap-1" htmlFor="colon-replacement">{t('colon-replacement')}
                        <SettingsInfoIcon headerKey="colon-replacement" textKey="auto-download-explanation"/>
                    </label>

                    <Controller
                    name="replacementStrategy"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomSelect id="colon-replacement" name={name} options={options} onChange={onChange} value ={value} />
                    )} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label className="text-[--fg-color] flex gap-1" htmlFor="episode-format">{t('standard-episode-format')}
                        <SettingsInfoIcon headerKey="standard-episode-format" textKey="standard-episode-format-explanation"/>
                    </label>

                    <Controller
                    name="episodeFormat"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomInput id="episode-format" name={name} onChange={onChange} value={value} />
                    )} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label className="text-[--fg-color] flex gap-1" htmlFor="podcast-format">{t('standard-podcast-format')}
                        <SettingsInfoIcon headerKey="standard-podcast-format" textKey="standard-podcast-format-explanation"/>
                    </label>

                    <Controller
                    name="podcastFormat"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomInput id="podcast-format" name={name} onChange={onChange} value={value} />
                    )} />
                </div>
                <fieldset className="xs:contents mb-4">
                    <legend className="self-start mb-2 xs:mb-0 text-[--fg-color]">{t('use-direct-paths')}</legend>

                    <div className="flex flex-col gap-2">
                        <div className="flex">
                            <Controller
                                name="directPaths"
                                control={control}
                                render={({ field: { name, onChange, value }}) => (
                                    <CustomCheckbox id="directPaths" name={name} onChange={onChange} value ={value} />
                                )} />
                        </div>
                    </div>
                </fieldset>
            </div>

            <CustomButtonPrimary className="float-right" type="submit">{t('save')}</CustomButtonPrimary>
        </form>
    )
}
