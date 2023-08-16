import { FC, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { enqueueSnackbar } from 'notistack'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { Podcast, setConfirmModalData, setPodcasts, podcastDeleted } from '../store/CommonSlice'
import { setModalOpen } from '../store/ModalSlice'
import { apiURL } from '../utils/Utilities'
import { CustomButtonSecondary } from './CustomButtonSecondary'

export const SettingsPodcastDelete: FC = () => {
    const dispatch = useAppDispatch()
    const podcasts = useAppSelector(state => state.common.podcasts)
    const { t } = useTranslation()

    useEffect(() => {
        if (podcasts.length === 0) {
            axios.get(apiURL + '/podcasts')
                .then((v) => {
                    dispatch(setPodcasts(v.data))
                })
        }
    }, [])

    const deletePodcast = (withFiles: boolean, podcast_id: number, p: Podcast) => {
        axios.delete(apiURL + '/podcast/' + podcast_id, { data: { delete_files: withFiles }})
            .then(() => {
                enqueueSnackbar(t('podcast-deleted', { name: p.name }), { variant: 'success' })
                dispatch(podcastDeleted(podcast_id))
            })
    }

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto_auto] items-center gap-6">
            {podcasts.map((p) => (
                <div className="grid grid-cols-1 xs:grid-cols-[auto_1fr] justify-items-start gap-2 lg:contents mb-4" key={p.id}>
                    <span className="xs:col-span-2 lg:col-span-1 text-[--fg-color]">{p.name}</span>

                    <CustomButtonSecondary onClick={() => {
                        dispatch(setConfirmModalData({
                            headerText: t('delete-podcast-with-files'),
                            onAccept:()=>{
                                deletePodcast(true, p.id, p)
                                dispatch(setModalOpen(false))
                            },
                            onReject: ()=>{
                                dispatch(setModalOpen(false))
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-with-files-body', {name: p.name})
                        }))
                        dispatch(setModalOpen(true))
                    }}>{t('delete-podcast-with-files')}</CustomButtonSecondary>

                    <CustomButtonSecondary onClick={() => {
                        dispatch(setConfirmModalData({
                            headerText: t('delete-podcast-without-files'),
                            onAccept:()=>{
                                deletePodcast(false, p.id, p)
                                dispatch(setModalOpen(false))
                            },
                            onReject: ()=>{
                                dispatch(setModalOpen(false))
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-without-files-body', {name: p.name})
                        }))
                        dispatch(setModalOpen(true))
                    }}>{t('delete-podcast-without-files')}</CustomButtonSecondary>
                </div>
            ))}
        </div>
    )
}
