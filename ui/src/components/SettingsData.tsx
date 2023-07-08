import {FC, useEffect, useState} from "react"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {useSnackbar} from "notistack"
import {apiURL} from "../utils/Utilities"
import {Setting} from "../models/Setting"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {CustomButtonSecondary} from "./CustomButtonSecondary"
import {CustomInput} from "./CustomInput"
import {Loading} from "./Loading"
import {Switcher} from "./Switcher"
import {SettingsInfoIcon} from "./SettingsInfoIcon";
import {PodcastInfoModal} from "./PodcastInfoModal";
import {InfoModal} from "./InfoModal";

type SettingsProps = {
    initialSettings: Setting
}

export const SettingsData:FC = ()=>{
    const [settings, setSettings] = useState<Setting>()

    /* Fetch existing settings */
    useEffect(()=>{
        axios.get(apiURL+"/settings").then((res:AxiosResponse<Setting>)=>{
            setSettings(res.data)
        })
    },[])

    if(settings === undefined){
        return <Loading />
    }

    return <Settings initialSettings={settings} />
}

export const Settings:FC<SettingsProps> = ({initialSettings}) => {
    const {t} = useTranslation()
    const {enqueueSnackbar} = useSnackbar()
    const [settings, setSettings] = useState<Setting>(initialSettings)

    return (
        <div>
            <div className="grid grid-cols-1 xs:grid-cols-[1fr_auto] items-center gap-2 xs:gap-6 mb-10 text-stone-900">
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <div>
                        <label className="mr-6" htmlFor="auto-cleanup">{t('auto-cleanup')}  </label>
                        <CustomButtonSecondary onClick={()=>{
                            axios.put(apiURL+"/settings/runcleanup")
                        }}>{t('run-cleanup')}</CustomButtonSecondary>
                    </div>
                    <Switcher checked={settings.autoCleanup} className="xs:justify-self-end" id="auto-cleanup" setChecked={()=>{
                        setSettings({...settings, autoCleanup: !settings?.autoCleanup})
                    }} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="days-to-keep" className="flex gap-1">{t('days-to-keep')}  <SettingsInfoIcon headerKey="days-to-keep" textKey="days-to-keep-explanation"/></label>
                    <CustomInput className="w-20" id="days-to-keep" onChange={(e)=>{
                        setSettings({...settings, autoCleanupDays: parseInt(e.target.value)})
                    }} type="number" value={settings.autoCleanupDays} />
                </div>

                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="auto-update" className="flex gap-1">{t('auto-update')} <SettingsInfoIcon headerKey="auto-update" textKey="auto-update-explanation"/></label>
                    <Switcher checked={settings.autoUpdate} className="xs:justify-self-end" id="auto-update" setChecked={()=>{
                        setSettings({...settings, autoUpdate: !settings?.autoUpdate})
                    }} />
                </div>



                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="auto-download" className="flex gap-1">{t('auto-download')} <SettingsInfoIcon headerKey="auto-download" textKey="auto-donwload-explanation"/></label>
                    <Switcher checked={settings.autoDownload} className="xs:justify-self-end" id="auto-download" setChecked={()=>{
                        setSettings({...settings, autoDownload: !settings?.autoDownload})
                    }} />
                </div>
                <div className="flex flex-col gap-2 xs:contents mb-4">
                    <label htmlFor="number-of-podcasts-to-download" className="flex gap-1">{t('number-of-podcasts-to-download')} <SettingsInfoIcon headerKey="number-of-podcasts-to-download" textKey="number-of-podcasts-to-download-explanation"/></label>
                    <CustomInput className="w-20" id="number-of-podcasts-to-download" onChange={(e)=>{
                        setSettings({...settings, podcastPrefill: parseInt(e.target.value)})
                    }} type="number" value={settings.podcastPrefill} />
                </div>
            </div>

            <CustomButtonPrimary className="float-right" onClick={()=>{
                axios.put(apiURL+"/settings", settings)
                    .then(()=>{
                        enqueueSnackbar(t('settings-saved'), {variant: "success"})
                    })
            }}>{t('save')}</CustomButtonPrimary>

        </div>
    )
}
