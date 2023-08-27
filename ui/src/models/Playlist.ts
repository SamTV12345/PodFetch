import {PodcastEpisode} from "../store/CommonSlice";
import {EpisodesWithOptionalTimeline} from "./EpisodesWithOptionalTimeline";

export type PlaylistDto = {
    id: number,
    name: string,
    items: EpisodesWithOptionalTimeline[]
}

export type PlaylistDtoPost = {
    name: string,
    items: PlaylistItem[]
}

export type PlaylistDtoPut = {
    id: number,
    name: string,
    items: PlaylistItem[]
}

export type PlaylistItem = {
    episode: number
}
