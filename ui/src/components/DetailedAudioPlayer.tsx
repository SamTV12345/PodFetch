import {FC, useEffect, useRef} from 'react'
import { createPortal } from 'react-dom'
import useCommon from '../store/CommonSlice'
import { removeHTML } from '../utils/Utilities'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PlayerTimeControls } from './PlayerTimeControls'
import { PlayerProgressBar } from './PlayerProgressBar'
import { PlayerVolumeSlider } from './PlayerVolumeSlider'
import 'material-symbols/outlined.css'
import useAudioPlayer from "../store/AudioPlayerSlice";
import {PodcastEpisodeChapterTable} from "./PodcastEpisodeChapterTable";
import {useTranslation} from "react-i18next";
import {isVideoUrl} from "../utils/audioPlayer";

type DetailedAudioPlayerProps = {
    audioAmplifier: AudioAmplifier | undefined,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier | undefined) => void
}

export const DetailedAudioPlayer: FC<DetailedAudioPlayerProps> = ({ audioAmplifier }) => {
    const setDetailedAudioPlayerOpen = useCommon(state=>state.setDetailedAudioPlayerOpen)
    const selectedTab = useCommon(state => state.detailedAudioPlayerTab)
    const setDetailedAudioPlayerTab = useCommon(state => state.setDetailedAudioPlayerTab)
    const currentPodcast = useAudioPlayer(state => state.currentPodcast)
    const currentPodcastEpisode = useAudioPlayer(state=>state.loadedPodcastEpisode)
    const {t} = useTranslation()
    const mediaUrl = currentPodcastEpisode?.podcastEpisode.local_url || currentPodcastEpisode?.podcastEpisode.url
    const isVideoEpisode = isVideoUrl(mediaUrl)
    const lastEpisodeIdRef = useRef<string | undefined>(undefined)

    useEffect(() => {
        const episodeId = currentPodcastEpisode?.podcastEpisode.episode_id
        if (!episodeId) {
            return
        }

        if (lastEpisodeIdRef.current !== episodeId) {
            lastEpisodeIdRef.current = episodeId
            if (isVideoEpisode) {
                setDetailedAudioPlayerTab('video')
            } else if (selectedTab === 'video') {
                setDetailedAudioPlayerTab('description')
            }
        }
    }, [currentPodcastEpisode?.podcastEpisode.episode_id, isVideoEpisode, selectedTab, setDetailedAudioPlayerTab])

    useEffect(() => {
        const mediaPlayer = document.getElementById('audio-player') as HTMLMediaElement | null
        const videoSlot = document.getElementById('video-player-slot')
        const parkingSlot = document.getElementById('media-player-parking-slot')
        if (!mediaPlayer || !videoSlot || !parkingSlot) {
            return
        }

        if (!isVideoEpisode) {
            if (mediaPlayer.parentElement !== parkingSlot) {
                parkingSlot.appendChild(mediaPlayer)
            }
            mediaPlayer.className = "hidden"
            return
        }

        if (mediaPlayer.parentElement !== videoSlot) {
            videoSlot.appendChild(mediaPlayer)
        }
        mediaPlayer.className = selectedTab === 'video'
            ? "w-full h-full rounded-xl bg-black object-contain"
            : "hidden"
    }, [isVideoEpisode, selectedTab])

    useEffect(() => {
        return () => {
            const player = document.getElementById('audio-player') as HTMLMediaElement | null
            const parking = document.getElementById('media-player-parking-slot')
            if (player && parking && player.parentElement !== parking) {
                parking.appendChild(player)
                player.className = "hidden"
            }
        }
    }, [])


    return createPortal(
        <div tabIndex={-1} aria-hidden="true" className="grid grid-rows-[1fr_auto] fixed inset-0 ui-surface md:h-full overflow-x-hidden overflow-y-auto z-30" onClick={event => event.stopPropagation()}>
        <span className="material-symbols-outlined absolute top-2 left-2 cursor-pointer text-4xl ui-text hover:ui-text-hover"
              onClick={() => setDetailedAudioPlayerOpen(false)}>close_fullscreen</span>

            {/* Episode information */}
            <div className="
        grid
        grid-cols-[1fr] grid-rows-[auto_1fr]
        md:grid-cols-[auto_1fr] md:grid-rows-1
        md:items-start gap-4 xs:gap-8 md:gap-10
        px-4 py-8 xs:px-8 md:px-12
        overflow-hidden
        ">
                {/* Thumbnail and titles */}
                <div className="flex flex-col xs:flex-row items-center gap-4 md:block h-full place-content-center">
                    <div className="aspect-square bg-center bg-cover md:mb-4 rounded-xl h-40 md:h-60 lg:h-80" style={{ backgroundImage: `url("${currentPodcastEpisode?.podcastEpisode.local_image_url}")` }}></div>

                    <div className="text-center xs:text-left">
                        <span className="block font-bold leading-tight mb-2 text-xl lg:text-2xl ui-text">{currentPodcastEpisode?.podcastEpisode.name}</span>
                        <span className="block lg:text-lg ui-text">{currentPodcast && currentPodcast.name}</span>
                    </div>
                </div>

                {/* Description with scroll */}
                <div className="">
                    <ul className="flex flex-wrap gap-2 border-b ui-border mb-6 ui-text-muted">
                        {isVideoEpisode && (
                            <li onClick={()=>setDetailedAudioPlayerTab('video')} className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'video' && 'border-b-2 ui-border-accent ui-text-accent'}`}>
                                {t('video')}
                            </li>
                        )}
                        <li onClick={()=>setDetailedAudioPlayerTab('description')} className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'description' && 'border-b-2 ui-border-accent ui-text-accent'}`}>
                            {t('description')}
                        </li>
                        <li onClick={()=>setDetailedAudioPlayerTab('chapters')} className={`cursor-pointer inline-block px-2 py-4 ${selectedTab === 'chapters' && 'border-b-2 ui-border-accent ui-text-accent'}`}>
                            {t('chapters')}
                        </li>
                    </ul>

                    <div className="overflow-y-auto overflow-x-hidden max-h-11/12 pr-2">
                    {currentPodcastEpisode && (
                        <>
                            {isVideoEpisode && (
                                <div className={selectedTab === 'video' ? "space-y-4 mb-4" : "hidden"}>
                                    <div
                                        id="video-player-slot"
                                        className="w-full max-w-[960px] aspect-video rounded-xl bg-black/40 border ui-border overflow-hidden"
                                    />
                                    <p className="text-sm ui-text-muted">
                                        {t('media-controls-hint')}
                                    </p>
                                </div>
                            )}
                        {selectedTab === 'description' ? (
                            <div className="text-sm md:text-base leading-7 ui-text max-w-full break-words [overflow-wrap:anywhere] [word-break:break-word] [&_a]:ui-text-accent [&_a:hover]:ui-text [&_p]:mb-4 [&_ul]:list-disc [&_ul]:ml-6 [&_ol]:list-decimal [&_ol]:ml-6 [&_li]:mb-1"
                                 dangerouslySetInnerHTML={currentPodcastEpisode?.podcastEpisode.description ? removeHTML(currentPodcastEpisode.podcastEpisode.description) : { __html: '' }} />
                        ): selectedTab === 'chapters' ? (<PodcastEpisodeChapterTable podcastEpisode={currentPodcastEpisode.podcastEpisode}/>) : null}
                        </>
                    )}
                    </div>
                </div>
            </div>

            {/* Player */}
            <div className="ui-surface px-2 xs:p-4">
                <PlayerProgressBar className="mb-2" currentPodcastEpisode={currentPodcastEpisode} />

                <div className="
                grid
                grid-col-1 xs:grid-cols-[0_1fr_12rem] md:grid-cols-[12rem_1fr_12rem]
                justify-items-center
                px-3 xs:px-4 md:px-10
            ">
                    <div></div>
                    <PlayerTimeControls currentPodcastEpisode={currentPodcastEpisode} />
                    <PlayerVolumeSlider audioAmplifier={audioAmplifier}/>
                </div>
            </div>

        </div>,document.getElementById('modal')!
    )
}
