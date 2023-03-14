import {Podcast, PodcastEpisode} from "../../store/CommonSlice";

export interface BroadcastMesage {
    type_of: string,
    message: string,


}

export interface PodcastAdded extends BroadcastMesage {
    podcast: Podcast
}

export interface PodcastEpisodeAdded extends BroadcastMesage {
    podcast_episode: PodcastEpisode
}

export interface PodcastEpisodeAdded extends BroadcastMesage {
    podcast_episode: PodcastEpisode
}

export interface PodcastEpisodesAdded extends BroadcastMesage {
    podcast_episodes: PodcastEpisode[]
}
