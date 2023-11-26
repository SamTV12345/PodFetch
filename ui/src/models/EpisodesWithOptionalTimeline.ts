import {PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "./PodcastWatchedModel";
import {Episode} from "./Episode";

export interface EpisodesWithOptionalTimeline {
    podcastEpisode: PodcastEpisode,
    podcastHistoryItem?: Episode
}
