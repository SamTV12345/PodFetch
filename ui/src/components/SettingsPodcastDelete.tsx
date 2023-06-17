import {FC, useEffect} from "react"
import {useTranslation} from "react-i18next"
import axios from "axios"
import {enqueueSnackbar} from "notistack"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {Podcast, setConfirmModalData, setPodcasts} from "../store/CommonSlice"
import {setModalOpen} from "../store/ModalSlice"
import {apiURL} from "../utils/Utilities"
import {ButtonSecondary} from './ButtonSecondary'

export const SettingsPodcastDelete:FC = () => {
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

    const deletePodcast = (withFiles: boolean, podcast_id: number, p: Podcast)=>{
        axios.delete(apiURL+"/podcast/"+podcast_id,{data: {delete_files: withFiles}})
            .then(()=>{
                enqueueSnackbar(t('podcast-deleted', {name: p.name}),{variant: "success"})
            })
    }

    return (
        <div className="grid grid-cols-[1fr_auto_auto] items-center gap-6">
            {podcasts.map((p) => (
                <div className="contents" key={p.id}>
                    <span className="text-stone-900">{p.name}</span>

                    <ButtonSecondary className="w-auto" onClick={()=>{
                        dispatch(setConfirmModalData({
                            headerText: t('delete-podcast-with-files'),
                            onAccept:()=>{
                                deletePodcast(true, p.id, p)
                            },
                            onReject: ()=>{
                                dispatch(setModalOpen(false))
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-with-files-body', {name: p.name})
                        }))
                        dispatch(setModalOpen(true))
                    }}>{t('delete-podcast-with-files')}</ButtonSecondary>

                    <ButtonSecondary onClick={()=>{
                        dispatch(setConfirmModalData({
                            headerText: t('delete-podcast-without-files'),
                            onAccept:()=>{
                                deletePodcast(false, p.id, p)
                            },
                            onReject: ()=>{
                                dispatch(setModalOpen(false))
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-without-files-body', {name: p.name})
                        }))
                        dispatch(setModalOpen(true))
                    }}>{t('delete-podcast-without-files')}</ButtonSecondary>
                </div>
            ))}
        </div>
    )
}
