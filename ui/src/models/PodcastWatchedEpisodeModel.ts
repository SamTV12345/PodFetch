import {Podcast, PodcastEpisode} from "../store/CommonSlice";

export interface PodcastWatchedEpisodeModel {
    id: number,
    podcastId: number,
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
