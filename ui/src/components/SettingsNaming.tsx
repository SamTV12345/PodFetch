import {FC, useEffect, useState} from "react"
import {useTranslation} from "react-i18next"
import {Controller, SubmitHandler, useForm} from "react-hook-form"
import axios, {AxiosResponse} from "axios"
import {enqueueSnackbar} from "notistack"
import {apiURL} from "../utils/Utilities"
import {Setting} from "../models/Setting"
import {UpdateNameSettings} from "../models/UpdateNameSettings"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {CustomSelect} from "../components/CustomSelect"
import {CustomInput} from "../components/CustomInput"
import {Loading} from "../components/Loading"
import { CustomCheckbox } from '../components/CustomCheckbox'

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

export const SettingsNaming:FC = ()=>{
    const [settings, setSettings] = useState<Setting>()

    /* Fetch existing settings */
    useEffect(()=>{
        axios.get(apiURL+"/settings").then((res:AxiosResponse<Setting>)=>{
            setSettings(res.data)
        })
    },[])

    if(settings === undefined){
        return <Loading/>
    }

    return <Settings intialSettings={settings}/>
}

const Settings:FC<SettingsProps> = ({intialSettings}) => {
    const {t} = useTranslation()

    const {control, formState: {}, handleSubmit, register} = useForm<UpdateNameSettings>({
        defaultValues: {
            replacementStrategy: intialSettings.replacementStrategy,
            episodeFormat: intialSettings.episodeFormat,
            replaceInvalidCharacters: intialSettings.replaceInvalidCharacters,
            /* TODO: Fix inconsistency - /api/v1/settings uses useExistingFilename whereas /api/v1/settings/name uses useExistingFilenames */
            useExistingFilenames: intialSettings.useExistingFilename,
            podcastFormat: intialSettings.podcastFormat
        }
    })

    const update_settings:SubmitHandler<UpdateNameSettings> = (data)=>{
        axios.put(apiURL+"/settings/name", data satisfies UpdateNameSettings)
            .then(()=>{
                enqueueSnackbar(t('settings-saved'), {variant: "success"})
            })
    }

    return (
        <form onSubmit={handleSubmit(update_settings)}>
            <div className="grid grid-cols-[1fr_auto] gap-6 mb-10">
                <fieldset className="contents">
                    <legend className="text-stone-900">{t('rename-podcasts')}</legend>

                    <div className="flex flex-col gap-2">
                        <div>
                            <Controller
                            name="useExistingFilenames"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomCheckbox id="use-existing-filenames" name={name} onChange={onChange} value ={value}/>
                            )}/>

                            <label className="ml-2 text-sm text-stone-600" htmlFor="use-existing-filenames">{t('use-existing-filenames')}</label>
                        </div>
                        <div>
                            <Controller
                            name="replaceInvalidCharacters"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomCheckbox id="replace-invalid-characters" name={name} onChange={onChange} value ={value}/>
                            )}/>

                            <label className="ml-2 text-sm text-stone-600" htmlFor="replace-invalid-characters">{t('replace-invalid-characters-description')}</label>
                        </div>
                    </div>
                </fieldset>

                <div className="contents">
                    <label className="text-stone-900" htmlFor="colon-replacement">{t('colon-replacement')}</label>

                    <Controller
                    name="replacementStrategy"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomSelect id="colon-replacement" name={name} options={options} onChange={onChange} value ={value}/>
                    )}/>
                </div>

                <div className="contents">
                    <label className="text-stone-900" htmlFor="episode-format">{t('standard-episode-format')}</label>

                    <Controller
                    name="episodeFormat"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomInput id="episode-format" name={name} onChange={onChange} value={value}/>
                    )}/>
                </div>

                <div className="contents">
                    <label className="text-stone-900" htmlFor="podcast-format">{t('standard-podcast-format')}</label>

                    <Controller
                    name="podcastFormat"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomInput id="podcast-format" name={name} onChange={onChange} value={value}/>
                    )}/>
                </div>
            </div>

            <CustomButtonPrimary className="float-right" type="submit">{t('save')}</CustomButtonPrimary>
        </form>
    )
}
