import {Podcast, PodcastEpisode} from "../../store/CommonSlice";
import {components} from "../../../schema";

export interface BroadcastMesage {
    type_of: string,
    message: string,


}

export interface PodcastAdded extends BroadcastMesage {
    podcast: components["schemas"]["PodcastDto"]
}

export interface PodcastRefreshed extends BroadcastMesage {
    podcast: components["schemas"]["PodcastDto"]
}
export interface PodcastEpisodeAdded extends BroadcastMesage, PodcastAdded {
    podcast_episode: components["schemas"]["PodcastEpisodeDto"]
}

export interface PodcastEpisodeDeleted extends BroadcastMesage, PodcastAdded {
    podcast_episode: components["schemas"]["PodcastEpisodeDto"]
}

export interface PodcastEpisodeAdded extends BroadcastMesage {
    podcast_episode: components["schemas"]["PodcastEpisodeDto"]
}

export interface PodcastEpisodesAdded extends BroadcastMesage, PodcastAdded {
    podcast_episodes: components["schemas"]["PodcastEpisodeDto"][]
}
