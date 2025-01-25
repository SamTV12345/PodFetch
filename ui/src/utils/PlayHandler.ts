import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "./Utilities";
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {components} from "../../schema";

export const handlePlayofEpisode = (response: components["schemas"]["EpisodeDto"], episode: EpisodesWithOptionalTimeline)=>{
    const handlePlayIfDownloaded = ()=>{
        episode.podcastEpisode.status
            ? useAudioPlayer.getState().setCurrentPodcastEpisode(preparePodcastEpisode(episode.podcastEpisode, response))
            : useAudioPlayer.getState().setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode.podcastEpisode, response))
        useAudioPlayer.getState().currentPodcast && useAudioPlayer.getState().setCurrentPodcast(useAudioPlayer.getState().currentPodcast!)
        useAudioPlayer.getState().setPlaying(true)
        return
    }
    if (response == null){
        handlePlayIfDownloaded()
        return
    }
    const playedPercentage = response.position! * 100 / episode.podcastEpisode.total_time
    if(playedPercentage < 95 || episode.podcastEpisode.total_time === 0){
        handlePlayIfDownloaded()
        return
    }
    else{
        useCommon.getState().setPodcastEpisodeAlreadyPlayed({
            podcastEpisode: episode,
            podcastWatchModel: response
        })
        useCommon.getState().setPodcastAlreadyPlayed(true)
    }
}
