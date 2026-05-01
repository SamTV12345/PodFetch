import {components} from "../../schema";

export interface ServerToClientEvents {
    offlineAvailable: (data: {
        podcast: components["schemas"]["PodcastDto"]
        type_of: string,
        podcast_episode: components["schemas"]["PodcastEpisodeDto"]
    })=>void,
    refreshedPodcast: (data: {
        message: string,
        podcast: components["schemas"]["PodcastDto"],
    })=>void,
    opmlError: (data: {
        message: string
    })=>void,
    opmlAdded: (data: {
        message: string
    })=>void,
    addedEpisodes: (data: {
        message: string,
        podcast: components["schemas"]["PodcastDto"],
        podcast_episodes: components["schemas"]["PodcastEpisodeDto"][]
    })=>void,
    deletedPodcastEpisodeLocally: (data: {
        podcast_episode: components["schemas"]["PodcastEpisodeDto"],
        message: string
    })=>void,
    addedPodcast: (data: {
        message: string,
        podcast: components["schemas"]["PodcastDto"]
    })=>void,
    "cast:status": (data: {
        status: {
            session_id: string,
            state: components["schemas"]["CastSessionState"],
            position_secs: number,
            volume: number,
            at: string,
        }
    })=>void,
    "cast:ended": (data: {
        session_id: string,
        reason: "stopped" | "finished" | "device_gone" | "error",
    })=>void,
}


export interface ClientToServerEvents {

}
