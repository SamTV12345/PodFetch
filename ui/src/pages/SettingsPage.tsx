import {useTranslation} from "react-i18next";
import {Switcher} from "../components/Switcher";
import {useEffect, useState} from "react";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Setting} from "../models/Setting";
import {useSnackbar} from "notistack";

export const SettingsPage = () => {
    const {t} = useTranslation()
    const [settings, setSettings] = useState<Setting>()
    const {enqueueSnackbar} = useSnackbar()
    useEffect(()=>{
        axios.get(apiURL+"/settings").then((res:AxiosResponse<Setting>)=>{
            setSettings(res.data)
        })
    },[])

    if(settings===undefined){
        return <div>Loading...</div>
    }


    return (
        <div className="p-6">
            <h1 className="text-2xl text-center font-bold">Settings</h1>

            <div className="bg-slate-900 rounded p-5 text-white">
                <div className="grid grid-cols-2 gap-5">
                    <div className="">{t('auto-cleanup')}</div>
                    <div><Switcher checked={settings.autoCleanup} setChecked={()=>{
                        setSettings({...settings, autoCleanup: !settings?.autoCleanup})
                    }}/></div>

                    <span className=" pt-2 pb-2">{t('days-to-keep')}</span>
                    <div><input type="number" className="bg-gray-600 p-2 rounded" value={settings.autoCleanupDays} onChange={(e)=>{
                        setSettings({...settings, autoCleanupDays: parseInt(e.target.value)})
                    }}/></div>
                    <div className="">
                        {t('auto-update')}
                    </div>
                    <div>
                        <Switcher checked={settings.autoUpdate} setChecked={()=>{
                            setSettings({...settings, autoUpdate: !settings?.autoUpdate})
                        }}/>
                    </div>
                    <div>
                        {t('auto-download')}
                    </div>
                    <div>
                        <Switcher checked={settings.autoDownload} setChecked={()=>{
                            setSettings({...settings, autoDownload: !settings?.autoDownload})
                        }}/>
                        <button className="bg-blue-600 rounded p-2 hover:bg-blue-500 ml-5" onClick={()=>{
                            axios.put(apiURL+"/settings/runcleanup")
                        }}>{t('run-cleanup')}</button>
                    </div>
                </div>
                <div className="flex">
                    <div className="flex-1"></div>
                    <button className=" p-2 bg-blue-600 rounded hover:bg-blue-500" onClick={()=>{
                        axios.put(apiURL+"/settings", settings)
                            .then(()=>{
                                enqueueSnackbar(t('settings-saved'), {variant: "success"})
                            })
                    }}>
                        {t('save')}
                    </button>
                </div>
            </div>
        </div>
    )
}
