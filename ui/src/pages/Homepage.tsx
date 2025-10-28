import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'
import { EpisodeCard } from '../components/EpisodeCard'
import { Heading2 } from '../components/Heading2'
import { PodcastEpisodeAlreadyPlayed } from '../components/PodcastEpisodeAlreadyPlayed'
import { LoadingPodcastCard } from '../components/ui/LoadingPodcastCard'
import { $api } from '../utils/http'

export const Homepage = () => {
	const { t } = useTranslation()
	const lastWatched = $api.useQuery(
		'get',
		'/api/v1/podcasts/episode/lastwatched',
	)
	const timeline = $api.useQuery('get', '/api/v1/podcasts/timeline', {
		params: {
			query: {
				favoredOnly: false,
				notListened: false,
				favoredEpisodes: false,
			},
		},
	})

	return (
		<>
			<PodcastEpisodeAlreadyPlayed />
			<div className="mb-8">
				<Heading2 className="mb-2">{t('last-listened')}</Heading2>

				<div
					className={`
                    scrollbox-x
                    pb-4 pt-8
                    w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                    xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                    md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
                `}
				>
					<div className="flex gap-8">
						{lastWatched.isLoading
							? Array.from({ length: 5 }).map((value, index, array) => (
									<LoadingPodcastCard key={index} />
								))
							: lastWatched.data?.map((v) => {
									return (
										<div
											className="basis-40 shrink-0 whitespace-normal"
											key={v.podcastEpisode.episode_id}
										>
											<EpisodeCard
												podcast={v.podcast}
												podcastEpisode={v.podcastEpisode}
												podcastHistory={v.episode}
											/>
										</div>
									)
								})}
					</div>
				</div>
			</div>
			<div>
				<div className="flex items-center gap-4 mb-2">
					<Heading2>{t('latest-episodes')}</Heading2>
					<Link
						className="text-sm text-(--accent-color) hover:text-(--accent-color-hover)"
						to="/timeline"
					>
						{t('view-more')}
					</Link>
				</div>

				<div
					className={`
                    scrollbox-x
                    pb-4 pt-8
                    w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                    xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                    md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
                `}
				>
					<div className="flex gap-8">
						{timeline.isLoading
							? Array.from({ length: 10 }).map((value, index, array) => (
									<LoadingPodcastCard key={index} />
								))
							: timeline.data?.data.map((episode) => {
									return (
										<div
											className="basis-40 shrink-0 whitespace-normal"
											key={episode.podcast_episode.episode_id}
										>
											<EpisodeCard
												podcast={episode.podcast}
												podcastEpisode={episode.podcast_episode}
												podcastHistory={episode.history}
											/>
										</div>
									)
								})}
					</div>
				</div>
			</div>
		</>
	)
}
