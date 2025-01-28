import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "./Utilities";
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {components} from "../../schema";

export const handlePlayofEpisode = (episode: components["schemas"]["PodcastEpisodeDto"], response?: components["schemas"]["EpisodeDto"])=>{
    const handlePlayIfDownloaded = ()=>{
        episode.status
            ? useAudioPlayer.getState().setCurrentPodcastEpisode(preparePodcastEpisode(episode, response))
            : useAudioPlayer.getState().setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(episode, response))
        useAudioPlayer.getState().currentPodcast && useAudioPlayer.getState().setCurrentPodcast(useAudioPlayer.getState().currentPodcast!)
        useAudioPlayer.getState().setPlaying(true)
        return
    }
    if (response == null){
        handlePlayIfDownloaded()
        return
    }
    const playedPercentage = response.position! * 100 / episode.total_time
    if(playedPercentage < 95 || episode.total_time === 0){
        handlePlayIfDownloaded()
        return
    }
    else{
        useCommon.getState().setPodcastEpisodeAlreadyPlayed({
            podcastEpisode: episode,
            podcastHistoryItem: response
        })
        useCommon.getState().setPodcastAlreadyPlayed(true)
    }
}
