import {useEffect} from "react";
import {useTranslation} from "react-i18next";
import {useParams} from "react-router-dom";
import {Heading2} from "../components/Heading2";
import {PodcastDetailItem} from "../components/PodcastDetailItem";
import {PodcastInfoModal} from "../components/PodcastInfoModal";
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import usePlaylist from "../store/PlaylistSlice";
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {startAudioPlayer} from "../utils/audioPlayer";
import {$api} from "../utils/http";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {CustomButtonSecondary} from "../components/CustomButtonSecondary";

export const PlaylistDetailPage = () => {
    const {t} = useTranslation()
    const params = useParams()
    const selectedPlaylist = usePlaylist(state => state.selectedPlaylist)
    const setSelectedPlaylist = usePlaylist(state => state.setSelectedPlaylist)
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const setSelectedEpisodeIndex = useAudioPlayer(state => state.setCurrentPodcastEpisode)
    const playlistQuery = $api.useQuery('get', '/api/v1/playlist/{playlist_id}', {
        params: {
            path: {
                playlist_id: String(params.id)
            }
        }
    }, {enabled: !!params.id})

    useEffect(() => {
        if (playlistQuery.data) {
            setSelectedPlaylist(playlistQuery.data)
        }
    }, [playlistQuery.data, setSelectedPlaylist])

    const playFromIndex = async (index: number) => {
        if (!selectedPlaylist?.items?.[index]) {
            return
        }
        const items = selectedPlaylist.items
        const startItem = items[index]!
        setSelectedEpisodes(items)
        setSelectedEpisodeIndex(index)
        await startAudioPlayer(
            startItem.podcastEpisode.local_url,
            startItem.podcastHistoryItem?.position ?? 0
        )
    }

    if (!selectedPlaylist) {
        return null
    }

    return (
        <div>
            <PodcastInfoModal/>
            <PodcastEpisodeAlreadyPlayed/>

            <div className="mb-6 rounded-xl border ui-border p-4">
                <Heading2 className="mb-2">{selectedPlaylist.name}</Heading2>
                <div className="text-sm ui-text-muted">
                    {t('item_other', {count: selectedPlaylist.items.length})}
                </div>
                <div className="mt-4 flex flex-wrap items-center gap-2">
                    <CustomButtonPrimary
                        onClick={() => {
                            void playFromIndex(0)
                        }}
                        disabled={selectedPlaylist.items.length === 0}
                    >
                        <span className="material-symbols-outlined leading-[0.875rem] mr-1">play_arrow</span>
                        {t('play-all')}
                    </CustomButtonPrimary>
                    <CustomButtonSecondary
                        onClick={() => {
                            void playFromIndex(0)
                        }}
                        disabled={selectedPlaylist.items.length === 0}
                    >
                        {t('restart')}
                    </CustomButtonSecondary>
                </div>
            </div>

            <Heading2 className="mb-8">{t('available-episodes')}</Heading2>
            {selectedPlaylist.items.map((episode, index) => {
                return (
                    <div key={episode.podcastEpisode.id} className="relative">
                        <PodcastDetailItem
                            onlyUnplayed={false}
                            episode={episode}
                            index={index}
                            currentEpisodes={selectedPlaylist.items}
                        />
                        <button
                            className="absolute right-0 top-0 text-xs ui-text-accent hover:ui-text-accent-hover"
                            onClick={() => {
                                void playFromIndex(index)
                            }}
                        >
                            {t('play-from-here')}
                        </button>
                    </div>
                )
            })}
        </div>
    )
}
