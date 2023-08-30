import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import {setInfoModalPodcastOpen, setPodcastAlreadyPlayed} from '../store/CommonSlice'
import { Heading2 } from './Heading2'
import 'material-symbols/outlined.css'
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {useMemo} from "react";
import {setCurrentPodcast, setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {store} from "../store/store";
import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";

export const PodcastEpisodeAlreadyPlayed = () => {
    const dispatch = useAppDispatch()
    const infoModalOpen = useAppSelector(state => state.common.podcastAlreadyPlayed)
    const selectedPodcastEpisode = useAppSelector(state => state.common.podcastEpisodeAlreadyPlayed)
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)
    const {t} = useTranslation()

    const percentagePlayed = useMemo(()=>{
        if(!selectedPodcastEpisode ||!selectedPodcastEpisode.podcastEpisode.podcastHistoryItem){
            return 0
        }
        return Math.round(selectedPodcastEpisode.podcastEpisode.podcastHistoryItem.watchedTime*100/selectedPodcastEpisode.podcastEpisode.podcastEpisode.total_time)
    }, [selectedPodcastEpisode?.podcastEpisode.podcastHistoryItem])
    const playedTime = useMemo(()=>{
        if(percentagePlayed === 0){
            return t('not-yet-played')
        }
        return t('podcast-episode-played',{
            percentage: percentagePlayed+"%"
        })
    },[selectedPodcastEpisode])

    return createPortal(
        <div
            id="defaultModal"
            tabIndex={-1}
            aria-hidden="true"
            onClick={() => dispatch(setPodcastAlreadyPlayed(false))}
            className={`fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden transition-opacity z-30
            ${!infoModalOpen && 'pointer-events-none'}
            ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`}
        >
            <div className={`relative bg-[--bg-color] max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] ${infoModalOpen ? 'opacity-100' : 'opacity-0'}`} onClick={e => e.stopPropagation()}>
                <button
                    type="button"
                    onClick={() => dispatch(setPodcastAlreadyPlayed(false))}
                    className="absolute top-4 right-4 bg-transparent"
                    data-modal-hide="defaultModal"
                >
                    <span className="material-symbols-outlined text-[--modal-close-color] hover:text-[--modal-close-color-hover]">close</span>
                    <span className="sr-only">Close modal</span>
                </button>

                <div className="text-white mb-5">
                    {t('you-already-listened',{
                        name:selectedPodcastEpisode?.podcastEpisode.podcastEpisode.name
                    })}
                </div>

                <div className="flex gap-3 float-right">
                    <CustomButtonSecondary onClick={()=>dispatch(setPodcastAlreadyPlayed(false))}>{t('cancel')}</CustomButtonSecondary>
                    <CustomButtonPrimary onClick={()=>{
                        dispatch(setCurrentPodcast(currentPodcast!))
                        dispatch(setPlaying(true))
                        if(!selectedPodcastEpisode) {
                            return
                        }

                        const watchedModel:PodcastWatchedModel = {
                        ...selectedPodcastEpisode.podcastWatchModel,
                            watchedTime: 0
                        }

                        selectedPodcastEpisode.podcastEpisode.podcastEpisode.status === 'D'
                            ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(selectedPodcastEpisode.podcastEpisode.podcastEpisode,
                                watchedModel )))
                            : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(selectedPodcastEpisode.podcastEpisode.podcastEpisode,
                                watchedModel)))
                    }}>{t('restart-playing')}</CustomButtonPrimary>
                </div>
            </div>
        </div>, document.getElementById('modal')!
    )
}
