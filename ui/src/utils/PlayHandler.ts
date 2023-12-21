import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "./Utilities";
import {Episode} from "../models/Episode";
import {AxiosResponse} from "axios";
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";

export const handlePlayofEpisode = (response: AxiosResponse<Episode>, episode: EpisodesWithOptionalTimeline)=>{
    const handlePlayIfDownloaded = ()=>{
        episode.podcastEpisode.status === 'D'
            ? useAudioPlayer.getState().setCurrentPodcastEpisode(preparePodcastEpisode(episode.podcastEpisode, response.data))
            : useAudioPlayer.getState().setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode.podcastEpisode, response.data))
        useAudioPlayer.getState().currentPodcast && useAudioPlayer.getState().setCurrentPodcast(useAudioPlayer.getState().currentPodcast!)
        useAudioPlayer.getState().setPlaying(true)
        return
    }
    if (response.data == null){
        handlePlayIfDownloaded()
        return
    }
    const playedPercentage = response.data.position * 100 / episode.podcastEpisode.total_time
    if(playedPercentage < 95 || episode.podcastEpisode.total_time === 0){
        handlePlayIfDownloaded()
        return
    }
    else{
        useCommon.getState().setPodcastEpisodeAlreadyPlayed({
            podcastEpisode: episode,
            podcastWatchModel: response.data
        })
        useCommon.getState().setPodcastAlreadyPlayed(true)
    }
}
