import { t } from 'i18next'
import { type FC, useMemo } from 'react'
import type { components } from '../../schema'
import useAudioPlayer from '../store/AudioPlayerSlice'
import useCommon from '../store/CommonSlice'
import { startAudioPlayer } from '../utils/audioPlayer'
import { $api } from '../utils/http'

type PodcastEpisodeChapterTableProps = {
	podcastEpisode: components['schemas']['PodcastEpisodeDto']
	className?: string
}

export const PodcastEpisodeChapterTable: FC<
	PodcastEpisodeChapterTableProps
> = ({ podcastEpisode, className }) => {
	const setSelectedEpisodes = useCommon((state) => state.setSelectedEpisodes)
	const setCurrentEpisodeIndex = useAudioPlayer(
		(state) => state.setCurrentPodcastEpisode,
	)
	const chapters = $api.useQuery(
		'get',
		'/api/v1/podcasts/episodes/{id}/chapters',
		{
			params: {
				path: {
					id: podcastEpisode.id,
				},
			},
		},
	)
	const _currentPodcast = $api.useQuery(
		'get',
		'/api/v1/podcasts/reverse/episode/{id}',
		{
			params: {
				path: {
					id: podcastEpisode.id,
				},
			},
		},
	)

	const timeslotDisplay = useMemo(() => {
		if (!chapters.data) {
			return []
		}
		return chapters.data.map((chapter) => {
			const start_minutes =
				chapter.startTime / 60 >= 1
					? `${Math.floor(chapter.startTime / 60)}:${(chapter.startTime % 60).toString().padStart(2, '0')}`
					: `0:${chapter.startTime.toString().padStart(2, '0')}`
			const end_minutes =
				chapter.endTime / 60 >= 1
					? `${Math.floor(chapter.endTime / 60)}:${(chapter.endTime % 60).toString().padStart(2, '0')}`
					: `0:${chapter.endTime.toString().padStart(2, '0')}`

			return `${start_minutes} - ${end_minutes}min`
		})
	}, [chapters])

	return (
		<div className={className}>
			<table className="text-left text-sm text-(--fg-color) w-full">
				<thead>
					<tr className="border-b border-(--border-color)">
						<th scope="col" className="pr-2 py-3">
							{t('title')}
						</th>
						<th scope="col" className="pr-2 py-3  hidden sm:table-cell">
							{t('timeslot')}
						</th>
						<th scope="col" className="px-2 py-3">
							{t('actions')}
						</th>
					</tr>
				</thead>
				<tbody>
					{chapters.data?.map((chapter, index) => (
						<tr className="border-b border-(--border-color)" key={chapter.id}>
							<td className="pr-2 py-4 break-words">{chapter.title}</td>
							<td className="pr-2 py-4 hidden sm:table-cell">
								{timeslotDisplay[index]}
							</td>
							<td className="pr-2 py-4">
								<span
									className={`
                    col-start-2 col-end-3 row-start-2 row-end-3
                    xs:col-start-3 xs:col-end-4 xs:row-start-1 xs:row-end-4
                    self-center material-symbols-outlined cursor-pointer !text-5xl text-(--fg-color) hover:text-(--fg-color-hover) active:scale-90
              `}
									onClick={async (e) => {
										// Prevent icon click from triggering info modal
										e.stopPropagation()
										setSelectedEpisodes([
											{
												podcastEpisode,
												podcastHistoryItem: null,
											},
										])
										setCurrentEpisodeIndex(0)
										await startAudioPlayer(
											podcastEpisode.local_url,
											chapter.startTime ?? 0,
										)
									}}
								>
									play_circle
								</span>
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	)
}
