import {components} from "../../schema";

export interface EpisodesWithOptionalTimeline {
    podcastEpisode: components["schemas"]["PodcastEpisodeDto"],
    podcastHistoryItem?: components["schemas"]["EpisodeDto"]
}
