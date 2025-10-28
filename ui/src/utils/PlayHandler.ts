import type { components } from '../../schema'
import useCommon from '../store/CommonSlice'
import { prepareOnlinePodcastEpisode, preparePodcastEpisode } from './Utilities'

export const handlePlayofEpisode = (
	episode: components['schemas']['PodcastEpisodeDto'],
	chapters: components['schemas']['PodcastChapterDto'][],
	response?: components['schemas']['EpisodeDto'],
) => {
	if (response == null) {
		return episode.status
			? preparePodcastEpisode(episode, chapters, response)
			: prepareOnlinePodcastEpisode(episode, chapters, response)
	}
	const playedPercentage = (response.position * 100) / episode.total_time
	if (playedPercentage < 95 || episode.total_time === 0) {
		return episode.status
			? preparePodcastEpisode(episode, chapters, response)
			: prepareOnlinePodcastEpisode(episode, chapters, response)
	} else {
		useCommon.getState().setPodcastEpisodeAlreadyPlayed({
			podcastEpisode: episode,
			podcastHistoryItem: response,
		})
		useCommon.getState().setPodcastAlreadyPlayed(true)
	}
}
