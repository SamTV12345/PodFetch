import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "./Utilities";
import useCommon from "../store/CommonSlice";
import {components} from "../../schema";

export const handlePlayofEpisode = (episode: components["schemas"]["PodcastEpisodeDto"],  chapters: components['schemas']['PodcastEpisodeChapter'][], response?: components["schemas"]["EpisodeDto"])=>{
    if (response == null){
        return episode.status
            ? preparePodcastEpisode(episode,chapters,  response)
            : prepareOnlinePodcastEpisode(episode,chapters, response )
    }
    const playedPercentage = response.position! * 100 / episode.total_time
    if(playedPercentage < 95 || episode.total_time === 0){
        return episode.status
            ? preparePodcastEpisode(episode,chapters,  response)
            : prepareOnlinePodcastEpisode(episode,chapters, response )
    }
    else{
        useCommon.getState().setPodcastEpisodeAlreadyPlayed({
            podcastEpisode: episode,
            podcastHistoryItem: response
        })
        useCommon.getState().setPodcastAlreadyPlayed(true)
    }
}
