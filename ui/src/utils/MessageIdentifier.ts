import {
    BroadcastMesage,
    PodcastAdded,
    PodcastEpisodeAdded,
    PodcastEpisodesAdded, PodcastRefreshed
} from "../models/messages/BroadcastMesage";

export const checkIfPodcastAdded = (message: BroadcastMesage): message is PodcastAdded => {
    return message.type_of === MessageType.ADD_PODCAST
}

export const checkIfPodcastEpisodeAdded = (message: BroadcastMesage): message is PodcastEpisodeAdded => {
    return message.type_of === MessageType.ADD_PODCAST_EPISODE
}

export const checkIfPodcastEpisodesAdded = (message: BroadcastMesage): message is PodcastAdded => {
    return message.type_of === MessageType.ADD_PODCAST_EPISODES
}

export const checkIfPodcastRefreshed = (message: BroadcastMesage): message is PodcastRefreshed => {
    return message.type_of === MessageType.REFRESH_PODCAST
}

enum MessageType {
    ADD_PODCAST = "AddPodcast",
    ADD_PODCAST_EPISODE = "AddPodcastEpisode",
    ADD_PODCAST_EPISODES = "AddPodcastEpisodes",
    REFRESH_PODCAST = "RefreshPodcast"
}
