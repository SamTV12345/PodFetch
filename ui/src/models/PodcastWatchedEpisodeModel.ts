import {Podcast, PodcastEpisode} from "../store/CommonSlice";

export interface PodcastWatchedEpisodeModel {
    id: string,
    podcastId: string,
    episodeId: string,
    url: string,
    name:string,
    date: string,
    imageUrl: string,
    watchedTime: number,
    totalTime: number,
    podcastEpisode: PodcastEpisode,
    podcast: Podcast
}
