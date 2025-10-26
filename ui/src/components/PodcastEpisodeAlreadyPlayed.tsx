import { createPortal } from 'react-dom'
import {Trans, useTranslation} from 'react-i18next'
import useCommon from '../store/CommonSlice'
import 'material-symbols/outlined.css'
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {useMemo} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {prepareOnlinePodcastEpisode, preparePodcastEpisode, removeHTML} from "../utils/Utilities";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {Episode} from "../models/Episode";
import {components} from "../../schema";
import {client} from "../utils/http";
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

    return createPortal(
        <div
            id="defaultModal"
            tabIndex={-1}
            aria-hidden="true"
            onClick={() => setPodcastAlreadyPlayed(false)}
            className={`fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden transition-opacity z-30
            ${!infoModalOpen && 'pointer-events-none'}
            ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`}
        >
            <div className={`relative bg-(--bg-color) max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`} onClick={e => e.stopPropagation()}>
                <button
                    type="button"
                    onClick={() => setPodcastAlreadyPlayed(false)}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal"
                >
                    <span className="material-symbols-outlined text-(--modal-close-color) hover:text-(--modal-close-color-hover)">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <div className="text-(--fg-color) mb-5">
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
                        await startAudioPlayer(selectedPodcastEpisode.podcastEpisode.url, 0)
                    }}>{t('restart-playing')}</CustomButtonPrimary>
                </div>
            </div>
        </div>, document.getElementById('modal')!
    )
}
