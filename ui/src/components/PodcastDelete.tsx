import {setConfirmModalData, setPodcasts} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useTranslation} from "react-i18next";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {enqueueSnackbar} from "notistack";
import {useEffect} from "react";

export const PodcastDelete = () => {
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    const {t} = useTranslation()

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

    return             <div className="bg-slate-900 rounded p-5 text-white">
        <h1 className="text-2xl text-center">{t('manage-podcasts')}</h1>
        <div className="mt-2">
            {
                podcasts.map(p=>
                    <div className="border-2 border-b-indigo-100 p-4">
                        <h2>{p.name}</h2>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-5">
                            <button className="p-2 bg-red-500" onClick={()=>{
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
                            }}>{t('delete-podcasts-without-files')}</button>
                            <button className="p-2 bg-red-500" onClick={()=>{
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
                            }}>{t('delete-podcasts-without-files')}</button>
                        </div>
                    </div>
                )
            }
        </div>
    </div>
}
