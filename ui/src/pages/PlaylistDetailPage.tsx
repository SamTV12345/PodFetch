import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'
import { Heading2 } from '../components/Heading2'
import { PodcastDetailItem } from '../components/PodcastDetailItem'
import { PodcastEpisodeAlreadyPlayed } from '../components/PodcastEpisodeAlreadyPlayed'
import { PodcastInfoModal } from '../components/PodcastInfoModal'
import usePlaylist from '../store/PlaylistSlice'
import { client } from '../utils/http'

export const PlaylistDetailPage = () => {
	const { t } = useTranslation()
	const params = useParams()
	const selectedPlaylist = usePlaylist((state) => state.selectedPlaylist)
	const setSelectedPlaylist = usePlaylist((state) => state.setSelectedPlaylist)

	useEffect(() => {
		client
			.GET('/api/v1/playlist/{playlist_id}', {
				params: {
					path: {
						playlist_id: String(params.id),
					},
				},
			})
			.then((response) => {
				if (response.data) {
					setSelectedPlaylist(response.data)
				}
			})
	}, [params.id, setSelectedPlaylist])

	return (
		selectedPlaylist && (
			<div>
				<Heading2 className="mb-8">{t('available-episodes')}</Heading2>
				<PodcastInfoModal />
				<PodcastEpisodeAlreadyPlayed />
				{selectedPlaylist.items.map((episode, index) => {
					return (
						<PodcastDetailItem
							onlyUnplayed={false}
							episode={episode}
							key={episode.podcastEpisode.id}
							index={index}
							currentEpisodes={selectedPlaylist.items}
						/>
					)
				})}
			</div>
		)
	)
}
