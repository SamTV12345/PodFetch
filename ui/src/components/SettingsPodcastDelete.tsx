import { FC, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { enqueueSnackbar } from 'notistack'
import useCommon, { Podcast} from '../store/CommonSlice'
import { apiURL } from '../utils/Utilities'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import useModal from "../store/ModalSlice";

export const SettingsPodcastDelete: FC = () => {
    const podcasts = useCommon(state => state.podcasts)
    const { t } = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)
    const setConfirmModalData  = useCommon(state => state.setConfirmModalData)
    const setPodcasts = useCommon(state => state.setPodcasts)
    const podcastDeleted = useCommon(state => state.podcastDeleted)


    useEffect(() => {
        if (podcasts.length === 0) {
            axios.get(apiURL + '/podcasts')
                .then((v) => {
                    setPodcasts(v.data)
                })
        }
    }, [])

    const deletePodcast = (withFiles: boolean, podcast_id: number, p: Podcast) => {
        axios.delete(apiURL + '/podcast/' + podcast_id, { data: { delete_files: withFiles }})
            .then(() => {
                enqueueSnackbar(t('podcast-deleted', { name: p.name }), { variant: 'success' })
                podcastDeleted(podcast_id)
            })
    }

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto_auto] items-center gap-6">
            {podcasts.map((p) => (
                <div className="grid grid-cols-1 xs:grid-cols-[auto_1fr] justify-items-start gap-2 lg:contents mb-4" key={p.id}>
                    <span className="xs:col-span-2 lg:col-span-1 text-[--fg-color]">{p.name}</span>

                    <CustomButtonSecondary onClick={() => {
                        setConfirmModalData({
                            headerText: t('delete-podcast-with-files'),
                            onAccept:()=>{
                                deletePodcast(true, p.id, p)
                                setModalOpen(false)
                            },
                            onReject: ()=>{
                               setModalOpen(false)
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-with-files-body', {name: p.name})
                        })
                        setModalOpen(true)
                    }}>{t('delete-podcast-with-files')}</CustomButtonSecondary>

                    <CustomButtonSecondary onClick={() => {
                        setConfirmModalData({
                            headerText: t('delete-podcast-without-files'),
                            onAccept:()=>{
                                deletePodcast(false, p.id, p)
                                setModalOpen(false)
                            },
                            onReject: ()=>{
                                setModalOpen(false)
                            },
                            acceptText: t('delete-podcast-confirm'),
                            rejectText: t('cancel'),
                            bodyText: t('delete-podcast-without-files-body', {name: p.name})
                        })
                        setModalOpen(true)
                    }}>{t('delete-podcast-without-files')}</CustomButtonSecondary>
                </div>
            ))}
        </div>
    )
}
