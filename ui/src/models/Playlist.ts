import {PodcastEpisode} from "../store/CommonSlice";

export type PlaylistDto = {
    id: number,
    name: string,
    items: PodcastEpisode[]
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
