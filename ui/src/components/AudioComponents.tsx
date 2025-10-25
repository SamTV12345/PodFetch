import {Activity, createRef, useEffect, useState} from 'react'
import { AudioAmplifier } from '../models/AudioAmplifier'
import { AudioPlayer } from './AudioPlayer'
import { DetailedAudioPlayer } from './DetailedAudioPlayer'
import useAudioPlayer from "../store/AudioPlayerSlice";
import useCommon from "../store/CommonSlice";
import {client} from "../utils/http";
import {handlePlayofEpisode} from "../utils/PlayHandler";

export const AudioComponents = () => {
    const ref = createRef<HTMLAudioElement>()
    const detailedAudioPodcastOpen = useCommon(state => state.detailedAudioPlayerOpen)
    const [audioAmplifier,setAudioAmplifier] = useState<AudioAmplifier>()
    const currentPodcastEpisodeIndex = useAudioPlayer(state=>state.currentPodcastEpisodeIndex)
    const currentPodcastEpisodes = useCommon(state=>state.selectedEpisodes)


    // Todo fix audio playing on episode change
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
                    const audioElement = (document.getElementById('hiddenaudio') as HTMLAudioElement)
                    audioElement.currentTime = retrievedPodcastEpisode.podcastHistoryItem?.position!
                    audioElement.load()
                    audioElement.play()
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
                    const audioElement = (document.getElementById('hiddenaudio') as HTMLAudioElement)
                    audioElement.currentTime = retrievedPodcastEpisode.podcastHistoryItem?.position!
                    audioElement.load()
                    audioElement.play()
                }
            }

        }
        if (currentPodcastEpisodeIndex) {
            loadEpisodeData()
        }
    }, [currentPodcastEpisodeIndex, currentPodcastEpisodes]);

    return (
        <>
            <AudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} />
            <Activity mode={detailedAudioPodcastOpen ? 'visible': 'hidden'}><DetailedAudioPlayer refItem={ref} audioAmplifier={audioAmplifier} setAudioAmplifier={setAudioAmplifier} /></Activity>
        </>
    )
}
