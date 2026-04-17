import * as Dialog from '@radix-ui/react-dialog'
import {Trans, useTranslation} from 'react-i18next'
import useCommon from '../store/CommonSlice'
import 'material-symbols/outlined.css'
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {useMemo} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {removeHTML} from "../utils/Utilities";
import {components} from "../../schema";
import {startAudioPlayer} from "../utils/audioPlayer";

export const PodcastEpisodeAlreadyPlayed = () => {
    const infoModalOpen = useCommon(state => state.podcastAlreadyPlayed)
    const selectedPodcastEpisode = useCommon(state => state.podcastEpisodeAlreadyPlayed)
    const setSelectedPodcastEpisodes = useCommon(state => state.setSelectedEpisodes)
    const selectedPodcastEpisodes = useCommon(state => state.selectedEpisodes)
    const {t} = useTranslation()
    const setCurrentPodcastEpisode = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const setPodcastAlreadyPlayed = useCommon(state => state.setPodcastAlreadyPlayed)

    const displayPodcastName = useMemo(() => {
        if(!selectedPodcastEpisode) {
            return {
                __html: ''
            }
        }
        return removeHTML(selectedPodcastEpisode?.podcastEpisode.name!)
    }, [selectedPodcastEpisode])

    return (
        <Dialog.Root open={infoModalOpen} onOpenChange={(open) => { if (!open) setPodcastAlreadyPlayed(false) }}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                            <span className="sr-only">{t('closeModal')}</span>
                        </Dialog.Close>

                        <Dialog.Title className="sr-only">{t('restart-playing')}</Dialog.Title>

                        <div className="ui-text mb-5">
                            <Trans t={t} i18nKey={'you-already-listened'} components={{
                                name: <span dangerouslySetInnerHTML={displayPodcastName}/>
                            }}/>
                        </div>

                        <div className="flex gap-3 float-right">
                            <CustomButtonSecondary onClick={()=>setPodcastAlreadyPlayed(false)}>{t('cancel')}</CustomButtonSecondary>
                            <CustomButtonPrimary onClick={async ()=>{
                                if(!selectedPodcastEpisode) {
                                    return
                                }

                                const watchedModel: components["schemas"]["EpisodeDto"] = {
                                ...selectedPodcastEpisode.podcastHistoryItem!,
                                    position: 0
                                }

                                let index = 0
                                const newSelectedEpisodes = selectedPodcastEpisodes.map(e=>{
                                    if(e.podcastEpisode.episode_id === selectedPodcastEpisode.podcastEpisode.episode_id) {
                                        return {
                                            ...e,
                                            podcastHistoryItem: watchedModel
                                        }
                                    }
                                    index += 1
                                    return e
                                })
                                setSelectedPodcastEpisodes(newSelectedEpisodes)
                                setCurrentPodcastEpisode(index)
                                await startAudioPlayer(selectedPodcastEpisode.podcastEpisode.local_url, 0)
                            }}>{t('restart-playing')}</CustomButtonPrimary>
                        </div>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
