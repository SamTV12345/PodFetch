import {Podcast, PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "./PodcastWatchedModel";
import {Episode} from "./Episode";

export type TimelineHATEOASModel = {
    data: TimeLineModel[],
    totalElements: number
}

export type TimeLineModel = {
    podcast: Podcast,
    podcast_episode: PodcastEpisode,
    history: Episode
}
