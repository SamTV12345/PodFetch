import {FC, useEffect, useState} from "react"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {useSnackbar} from "notistack"
import {apiURL} from "../utils/Utilities"
import {Setting} from "../models/Setting"
import {ButtonPrimary} from "./ButtonPrimary"
import {ButtonSecondary} from "./ButtonSecondary"
import {CustomInput} from "./CustomInput"
import {Loading} from "../components/Loading"
import {Switcher} from "./Switcher"

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
        return <Loading/>
    }

    return <Settings initialSettings={settings}/>
}

export const Settings:FC<SettingsProps> = ({initialSettings}) => {
    const {t} = useTranslation()
    const {enqueueSnackbar} = useSnackbar()
    const [settings, setSettings] = useState<Setting>(initialSettings)

    return (
        <div>
            <div className="content-stretch grid grid-cols-1 sm:grid-cols-[1fr_auto] items-center gap-6 mb-10 text-stone-900">
                <div>
                    <span>{t('auto-cleanup')}</span>
                    <ButtonSecondary className=" ml-6" onClick={()=>{
                        axios.put(apiURL+"/settings/runcleanup")
                    }}>{t('run-cleanup')}</ButtonSecondary>
                </div>
                <div className="text-right">
                    <Switcher checked={settings.autoCleanup} setChecked={()=>{
                        setSettings({...settings, autoCleanup: !settings?.autoCleanup})
                    }}/>
                </div>

                <span>{t('days-to-keep')}</span>
                <div className="text-right">
                    <CustomInput type="number" className="w-20" value={settings.autoCleanupDays} onChange={(e)=>{
                        setSettings({...settings, autoCleanupDays: parseInt(e.target.value)})
                    }}/>
                </div>

                <span>{t('auto-update')}</span>
                <div className="text-right">
                    <Switcher checked={settings.autoUpdate} setChecked={()=>{
                        setSettings({...settings, autoUpdate: !settings?.autoUpdate})
                    }}/>
                </div>

                <span>{t('number-of-podcasts-to-download')}</span>
                <div className="text-right">
                    <CustomInput type="number" className="w-20" value={settings.podcastPrefill} onChange={(e)=>{
                        setSettings({...settings, podcastPrefill: parseInt(e.target.value)})
                    }}/>
                </div>

                <span>{t('auto-download')}</span>
                <div className="text-right">
                    <Switcher checked={settings.autoDownload} setChecked={()=>{
                        setSettings({...settings, autoDownload: !settings?.autoDownload})
                    }}/>
                </div>   
            </div>

            <ButtonPrimary className="float-right" onClick={()=>{
                axios.put(apiURL+"/settings", settings)
                    .then(()=>{
                        enqueueSnackbar(t('settings-saved'), {variant: "success"})
                    })
            }}>{t('save')}</ButtonPrimary>
        </div>
    )
}
