import { useQueryClient } from '@tanstack/react-query'
import type { FC } from 'react'
import { Waypoint } from 'react-waypoint'
import type { components } from '../../schema'
import useCommon from '../store/CommonSlice'
import { $api, client } from '../utils/http'
import { EpisodeCard } from './EpisodeCard'

type TimelineEpisodeProps = {
	podcastEpisode: components['schemas']['PodcastEpisodeDto']
	index: number
	timelineLength: number
	totalLength: number
	timeLineEpisodes: components['schemas']['TimeLinePodcastItem']
	notListened: boolean
	podcastHistoryItem?: components['schemas']['EpisodeDto']
	favoredEpisodes: boolean
	podcast: components['schemas']['PodcastDto']
}

export const TimelineEpisode: FC<TimelineEpisodeProps> = ({
	podcastEpisode,
	podcastHistoryItem,
	notListened,
	index,
	timeLineEpisodes,
	favoredEpisodes,
	podcast,
}) => {
	const filters = $api.useQuery('get', '/api/v1/podcasts/filter')
	const queryClient = useQueryClient()

	return (
		<>
			<EpisodeCard
				podcastHistory={podcastHistoryItem}
				podcast={podcast}
				podcastEpisode={podcastEpisode}
			/>

			{
				/*Infinite scroll */
				timeLineEpisodes.data.length === index + 1 && (
					<Waypoint
						key={index + 'waypoint'}
						onEnter={() => {
							client
								.GET('/api/v1/podcasts/timeline', {
									params: {
										query: {
											lastTimestamp: podcastEpisode.date_of_recording,
											favoredOnly: filters.data?.onlyFavored ?? false,
											notListened: notListened,
											favoredEpisodes,
										},
									},
								})
								.then((resp) => {
									for (const cache of queryClient.getQueryCache().getAll()) {
										if (
											cache.queryKey[0] === 'get' &&
											(cache.queryKey[1] as string) ===
												'/api/v1/podcasts/timeline'
										) {
											queryClient.setQueryData(
												cache.queryKey,
												(
													oldData:
														| components['schemas']['TimeLinePodcastItem']
														| undefined,
												) => {
													if (oldData) {
														return {
															...oldData,
															data: [
																...oldData.data,
																...(resp.data?.data ?? []),
															],
															total:
																resp.data?.totalElements ??
																oldData.totalElements,
														}
													}
													return resp.data
												},
											)
										}
									}
								})
						}}
					/>
				)
			}
		</>
	)
}
