import {BroadcastMesage, PodcastAdded} from "../models/messages/BroadcastMesage";

export const checkIfPodcastAdded = (message: BroadcastMesage): message is PodcastAdded => {
    return message.type_of === "ADD_PODCAST"
}

export const checkIfPodcastEpisodeAdded = (message: BroadcastMesage): message is PodcastAdded => {
    return message.type_of === "ADD_PODCAST_EPISODE"
}

export const checkIfPodcastEpisodesAdded = (message: BroadcastMesage): message is PodcastAdded => {
    return message.type_of === "ADD_PODCAST_EPISODES"
}
