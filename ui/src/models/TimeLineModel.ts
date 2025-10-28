import type { Podcast, PodcastEpisode } from '../store/CommonSlice'
import type { Episode } from './Episode'

export type TimelineHATEOASModel = {
	data: TimeLineModel[]
	totalElements: number
}

export type TimeLineModel = {
	podcast: Podcast
	podcast_episode: PodcastEpisode
	history: Episode
}
