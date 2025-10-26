import {Activity, createRef, useEffect, useState} from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { AudioPlayer } from './AudioPlayer'
import { DetailedAudioPlayer } from './DetailedAudioPlayer'
import useAudioPlayer from "../store/AudioPlayerSlice";
import useCommon from "../store/CommonSlice";
import {client} from "../utils/http";
import {handlePlayofEpisode} from "../utils/PlayHandler";
import {getAudioPlayer} from "../utils/audioPlayer";

export const AudioComponents = () => {
    const detailedAudioPodcastOpen = useCommon(state => state.detailedAudioPlayerOpen)
    const [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()
    const currentPodcastEpisodeIndex = useAudioPlayer(state=>state.currentPodcastEpisodeIndex)
    const currentPodcastEpisodes = useCommon(state=>state.selectedEpisodes)


    useEffect(() => {
        async function loadEpisodeData() {
            if (!currentPodcastEpisodes[currentPodcastEpisodeIndex!]) {
                return ;
            }
            const currentPodcastEpisode = currentPodcastEpisodes[currentPodcastEpisodeIndex!]!;
            try {
                const respForPodcast = await client.GET("/api/v1/podcasts/episode/{id}", {
                    params: { path: { id: currentPodcastEpisode.podcastEpisode.episode_id } }
                });
                const chaptersOfEpisode = await client.GET("/api/v1/podcasts/episodes/{id}/chapters", {
                    params: { path: { id: currentPodcastEpisode.podcastEpisode.id } }
                });

                const retrievedPodcastEpisode = handlePlayofEpisode(currentPodcastEpisode.podcastEpisode, chaptersOfEpisode.data ?? [], respForPodcast.data!);
                if (retrievedPodcastEpisode) {
                    useAudioPlayer.setState({
                        loadedPodcastEpisode: retrievedPodcastEpisode
                    })
                }
            } catch (e) {
                const chaptersOfEpisode = await client.GET("/api/v1/podcasts/episodes/{id}/chapters", {
                    params: { path: { id: currentPodcastEpisode.podcastEpisode.id } }
                });
                const retrievedPodcastEpisode = handlePlayofEpisode(currentPodcastEpisode.podcastEpisode, chaptersOfEpisode.data ?? [], undefined);
                if (retrievedPodcastEpisode) {
                    useAudioPlayer.setState({
                        loadedPodcastEpisode: retrievedPodcastEpisode
                    })
                }
            }
        }
        if (currentPodcastEpisodeIndex != null) {
            loadEpisodeData()
        }
    }, [currentPodcastEpisodeIndex, currentPodcastEpisodes]);

    return (
        <>
            <AudioPlayer audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />
            <Activity mode={detailedAudioPodcastOpen ? 'visible': 'hidden'}><DetailedAudioPlayer audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} /></Activity>
        </>
    )
}
