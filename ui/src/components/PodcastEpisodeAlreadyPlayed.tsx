import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import {Trans, useTranslation} from 'react-i18next'
import useCommon from '../store/CommonSlice'
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
        <Dialog open={infoModalOpen} onOpenChange={(open) => { if (!open) setPodcastAlreadyPlayed(false) }}>
            <DialogContent className="max-w-2xl">
                <DialogTitle className="sr-only">{t('restart-playing')}</DialogTitle>

                <div className="ui-text mb-5">
                    <Trans t={t} i18nKey={'you-already-listened'} components={{
                        name: <span dangerouslySetInnerHTML={displayPodcastName}/>
                    }}/>
                </div>

                <div className="flex gap-3 float-right">
                    <CustomButtonSecondary onClick={() => setPodcastAlreadyPlayed(false)}>{t('cancel')}</CustomButtonSecondary>
                    <CustomButtonPrimary onClick={async () => {
                        if (!selectedPodcastEpisode) {
                            return
                        }

                        const watchedModel: components["schemas"]["EpisodeDto"] = {
                            ...selectedPodcastEpisode.podcastHistoryItem!,
                            position: 0
                        }

                        let index = 0
                        const newSelectedEpisodes = selectedPodcastEpisodes.map(e => {
                            if (e.podcastEpisode.episode_id === selectedPodcastEpisode.podcastEpisode.episode_id) {
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
            </DialogContent>
        </Dialog>
    )
}
