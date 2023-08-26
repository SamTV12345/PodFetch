import {PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "./PodcastWatchedModel";

export interface EpisodesWithOptionalTimeline {
    podcastEpisode: PodcastEpisode,
    podcastHistoryItem?: PodcastWatchedModel
}
