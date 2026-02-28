import {FC, RefObject, useEffect, useState} from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { PlayerTimeControls } from './PlayerTimeControls'
import { PlayerEpisodeInfo } from './PlayerEpisodeInfo'
import { PlayerProgressBar } from './PlayerProgressBar'
import { PlayerVolumeSlider } from './PlayerVolumeSlider'
import useAudioPlayer from "../store/AudioPlayerSlice";
import {Podcast} from "../store/CommonSlice";
import {$api} from "../utils/http";
import {components} from "../../schema";

type DrawerAudioPlayerProps = {
    audioAmplifier: AudioAmplifier | undefined
}

export const DrawerAudioPlayer: FC<DrawerAudioPlayerProps> = ({ audioAmplifier }) => {
    const podcastEpisode = useAudioPlayer(state => state.loadedPodcastEpisode)
    const podcast = $api.useQuery('get','/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: String(podcastEpisode?.podcastEpisode.podcast_id)
            }
        }
    })

    return (
        <div className="
            fixed bottom-0 left-0 right-0 z-50
            col-span-2 grid
            sm:grid-cols-[10rem_1fr_8rem] md:grid-cols-[16rem_1fr_10rem] lg:grid-cols-[20rem_1fr_12rem]
            justify-items-center sm:justify-items-stretch
            gap-2 sm:gap-4 lg:gap-10
            p-4 lg:pr-8
            ui-surface ui-player-drawer-shadow
        ">
            {podcast.data && <PlayerEpisodeInfo podcast={podcast.data} podcastEpisode={podcastEpisode?.podcastEpisode}/>}

            <div className="flex flex-col gap-2 w-full">
                <PlayerTimeControls currentPodcastEpisode={podcastEpisode}/>
                <PlayerProgressBar currentPodcastEpisode={podcastEpisode}/>
            </div>

            <PlayerVolumeSlider audioAmplifier={audioAmplifier}/>
        </div>
    )
}
