export interface PodcastWatchedEpisodeModel {
    id: number,
    podcastId: number,
    episodeId: string,
    url: string,
    date: string,
    imageUrl: string,
    watchedTime: number
}
