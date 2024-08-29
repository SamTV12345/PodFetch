import {FC, Fragment, useEffect, useState} from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { enqueueSnackbar } from 'notistack'
import useCommon, { Podcast} from '../store/CommonSlice'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import useModal from "../store/ModalSlice";
import {CustomCheckbox} from "./CustomCheckbox";

export const SettingsPodcastDelete: FC = () => {
    const podcasts = useCommon(state => state.podcasts)
    const { t } = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)
    const setConfirmModalData  = useCommon(state => state.setConfirmModalData)
    const setPodcasts = useCommon(state => state.setPodcasts)
    const podcastDeleted = useCommon(state => state.podcastDeleted)
    const [selectedPodcasts, setSelectedPodcasts] = useState<Podcast[]>([])

    useEffect(() => {
        if (podcasts.length === 0) {
            axios.get('/podcasts')
                .then((v) => {
                    setPodcasts(v.data)
                })
        }
    }, [])

    const deletePodcast = (withFiles: boolean) => {
        selectedPodcasts.forEach(p=>{
            axios.delete( '/podcast/' + p.id, { data: { delete_files: withFiles }})
                .then(() => {
                    enqueueSnackbar(t('podcast-deleted', { name: p.name }), { variant: 'success' })
                    podcastDeleted(p.id)
                })
        })

    }

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_auto_auto] items-center gap-6">
            <CustomCheckbox value={selectedPodcasts.length === podcasts.length} onChange={(v)=>{
                if (v.valueOf() === true) {
                    setSelectedPodcasts(podcasts)
                } else {
                    setSelectedPodcasts([])
                }
            }}/>

            <CustomButtonSecondary disabled={selectedPodcasts.length === 0} onClick={() => {
                setConfirmModalData({
                    headerText: t('delete-podcast-with-files'),
                    onAccept:()=>{
                        deletePodcast(true)
                        setModalOpen(false)
                    },
                    onReject: ()=>{
                        setModalOpen(false)
                    },
                    acceptText: t('delete-podcast-confirm'),
                    rejectText: t('cancel'),
                    bodyText: t('delete-podcast-with-files-body', {name: [selectedPodcasts.map(a=>a.name).join(', ')]})
                })
                setModalOpen(true)
            }}>{t('delete-podcast-with-files')}</CustomButtonSecondary>

            <CustomButtonSecondary disabled={selectedPodcasts.length === 0} onClick={() => {
                setConfirmModalData({
                    headerText: t('delete-podcast-without-files'),
                    onAccept:()=>{
                        deletePodcast(false)
                        setModalOpen(false)
                    },
                    onReject: ()=>{
                        setModalOpen(false)
                    },
                    acceptText: t('delete-podcast-confirm'),
                    rejectText: t('cancel'),
                    bodyText: t('delete-podcast-without-files-body', {name: [selectedPodcasts.map(a=>a.name).join(', ')]})
                })
                setModalOpen(true)
            }}>{t('delete-podcast-without-files')}</CustomButtonSecondary>
            <hr className="col-span-1 lg:col-span-3"/>
            {podcasts.map((p) => (

                <Fragment key={p.id}>
                   <CustomCheckbox value={selectedPodcasts.includes(p)} onChange={(v)=>{
                        let isChecked = v.valueOf() as boolean

                        if (isChecked) {
                            if (!selectedPodcasts.includes(p)) {
                                setSelectedPodcasts([...selectedPodcasts, p])
                            }
                        } else {
                            setSelectedPodcasts(selectedPodcasts.filter(pod=>pod !== p))
                        }
                    }}/>
                    <span className="xs:col-span-2 lg:col-span-2 text-[--fg-color]">{p.name}</span>
                        </Fragment>
            ))}
        </div>
    )
}
