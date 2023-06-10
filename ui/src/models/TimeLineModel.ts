import {Podcast, PodcastEpisode} from "../store/CommonSlice";

export type TimelineHATEOASModel = {
    data: TimeLineModel[],
    totalElements: number
}

export type TimeLineModel = {
    podcast: Podcast,
    podcast_episode: PodcastEpisode
}
