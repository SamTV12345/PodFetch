export interface PodcastWatchedEpisodeModel {
    id: number,
    podcastId: number,
    episodeId: string,
    url: string,
    name:string,
    date: string,
    imageUrl: string,
    watchedTime: number,
    totalTime: number
}
