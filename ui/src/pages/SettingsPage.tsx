import {useTranslation} from "react-i18next";
import {Switcher} from "../components/Switcher";
import {useEffect, useState} from "react";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Setting} from "../models/Setting";
import {useSnackbar} from "notistack";
import {Loading} from "../components/Loading";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setConfirmModalData, setPodcasts} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";
import {ConfirmModal} from "../components/ConfirmModal";

export const SettingsPage = () => {
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const [settings, setSettings] = useState<Setting>()
    const {enqueueSnackbar} = useSnackbar()
    const podcasts = useAppSelector(state=>state.common.podcasts)

    useEffect(()=>{
        axios.get(apiURL+"/settings").then((res:AxiosResponse<Setting>)=>{
            setSettings(res.data)
        })
    },[])

    useEffect(()=>{
        if(podcasts.length===0){
            axios.get(apiURL+"/podcasts")
                .then((v)=>{
                    dispatch(setPodcasts(v.data))
                })
        }
    },[])

    const deletePodcast = (withFiles:boolean, podcast_id: number)=>{
        axios.delete(apiURL+"/podcast/"+podcast_id,{data: {delete_files: withFiles}})
            .then(()=>{
                enqueueSnackbar(t('podcast-deleted'),{variant: "success"})
            })
    }

    if(settings===undefined){
        return <Loading/>
    }


    return (
        <div className="p-6">
            <ConfirmModal/>
            <h1 className="text-2xl text-center font-bold">{t('settings')}</h1>
            <div className="grid gap-5">
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

            <div className="bg-slate-900 rounded p-5 text-white">
                <h1 className="text-2xl text-center">Podcasts verwalten</h1>
                <div className="mt-2">
                    {
                      podcasts.map(p=>
                           <div className="border-2 border-b-indigo-100 p-4">
                               <h2>{p.name}</h2>
                               <div className="grid grid-cols-2 gap-5">
                                   <button className="bg-red-500" onClick={()=>{
                                       dispatch(setConfirmModalData({
                                           headerText: t('delete-podcast-with-files'),
                                           onAccept:()=>{
                                               deletePodcast(true, p.id)
                                           },
                                           onReject: ()=>{
                                               dispatch(setModalOpen(false))
                                           },
                                           acceptText: t('delete-podcast'),
                                           rejectText: t('cancel'),
                                           bodyText: t('delete-podcast-with-files-body', {name: p.name})
                                       }))
                                       dispatch(setModalOpen(true))
                                   }}>Delete podcast with files</button>
                                   <button className="bg-red-500" onClick={()=>{
                                       dispatch(setConfirmModalData({
                                           headerText: t('delete-podcast-without-files'),
                                           onAccept:()=>{
                                               deletePodcast(false, p.id)
                                           },
                                           onReject: ()=>{
                                               dispatch(setModalOpen(false))
                                           },
                                           acceptText: t('delete-podcast'),
                                           rejectText: t('cancel'),
                                           bodyText: t('delete-podcast-without-files-body', {name: p.name})
                                       }))
                                       dispatch(setModalOpen(true))
                                   }}>Delete podcast without deleting files</button>
                               </div>
                           </div>
                      )
                    }
                </div>
            </div>
        </div>
        </div>
    )
}
