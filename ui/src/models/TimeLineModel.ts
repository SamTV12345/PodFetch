import {Podcast, PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "./PodcastWatchedModel";

export type TimelineHATEOASModel = {
    data: TimeLineModel[],
    totalElements: number
}

export type TimeLineModel = {
    podcast: Podcast,
    podcast_episode: PodcastEpisode,
    history: PodcastWatchedModel
}
